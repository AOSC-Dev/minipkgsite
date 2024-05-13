use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use eyre::{eyre, OptionExt, Result};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::{debug, info};
use walkdir::WalkDir;

pub struct Abbs {
    conn: MultiplexedConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    desc: String,
    path: String,
    section: String,
    deps: Vec<String>,
    build_deps: Vec<String>,
    pkgbreak: Vec<PkgStmt>,
    pkgrecom: Vec<String>,
    provides: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PkgStmt {
    name: String,
    comp: String,
    version: String,
}

impl Abbs {
    const TABLE_NAME_STABLE: &'static str = "aosc-packages-stable";
    pub fn new(client: MultiplexedConnection) -> Result<Abbs> {
        Ok(Abbs { conn: client })
    }

    pub async fn update_all(&mut self, git_path: PathBuf, first_time: bool) -> Result<()> {
        let old = head_commit(&git_path).await?;

        let out = Command::new("git")
            .arg("pull")
            .current_dir(&git_path)
            .output()
            .await?;

        info!(
            "git pull stdout: {}",
            String::from_utf8_lossy(&out.stdout).trim()
        );
        info!(
            "git pull stderr: {}",
            String::from_utf8_lossy(&out.stdout).trim()
        );

        let new = head_commit(&git_path).await?;

        if old == new && !first_time {
            info!("No need to update tree");
            return Ok(());
        }

        info!("Updating tree ...");

        let res = tokio::task::spawn_blocking(move || collection_packages(git_path)).await??;

        for i in res {
            debug!("insert {}", i.name);
            self.conn
                .set(
                    format!("{}:{}", Self::TABLE_NAME_STABLE, i.name),
                    serde_json::to_string(&i)?,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn get(&mut self, name: &str) -> Result<Package> {
        let res = self
            .conn
            .get::<&str, String>(&format!("{}:{name}", Self::TABLE_NAME_STABLE))
            .await?;

        Ok(serde_json::from_str(&res)?)
    }

    pub async fn all(&mut self) -> Result<Vec<Package>> {
        let mut res = vec![];
        for i in self.query_key_cmd_to_vec("*").await? {
            res.push(self.get(&i).await?);
        }

        Ok(res)
    }

    pub async fn search_by_stars(&mut self, stars: &str) -> Result<Vec<String>> {
        self.query_key_cmd_to_vec(&format!("{stars}*")).await
    }

    async fn query_key_cmd_to_vec(&mut self, query: &str) -> Result<Vec<String>, eyre::Error> {
        let mut keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{}:{query}", Self::TABLE_NAME_STABLE))
            .query_async(&mut self.conn)
            .await?;

        let prefix = format!("{}:", Self::TABLE_NAME_STABLE);

        for i in &mut keys {
            *i = i.strip_prefix(&prefix).unwrap().to_string();
        }

        Ok(keys)
    }
}

async fn head_commit(git_path: &Path) -> Result<String> {
    let cmd = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(git_path)
        .output()
        .await?;

    let commit = String::from_utf8_lossy(&cmd.stdout).trim().to_string();

    Ok(commit)
}

fn collection_packages(git_path: PathBuf) -> Result<Vec<Package>> {
    info!("Running git pull and insert to database ...");
    let dir = WalkDir::new(&git_path).max_depth(2).min_depth(2);
    let mut res = vec![];

    for i in dir {
        let mut ver = String::new();
        let i = i?;

        let p = i.path().display().to_string();

        if p.contains(".git") || p.contains("groups") {
            continue;
        }

        let mut context = HashMap::new();
        let path = i.path().strip_prefix(&git_path)?.display().to_string();

        let n = i.file_name().to_string_lossy();

        debug!("scanning {}", n);

        let spec = i.path().join("spec");

        if spec.is_file() {
            let c = std::fs::read_to_string(spec)?;
            parse_abbs_file_apml(&c, &mut context).unwrap_or_else(|e| {
                debug!("Failed to parse pkg {n} file using apml: {e}");
                more_parse(&c, &mut context)
            });
            if let Some(v) = context.get("VER") {
                ver.push_str(v);
            }

            if let Some(v) = context.get("REL") {
                ver.push('-');
                ver.push_str(v);
            }
        }

        let defines = i.path().join("autobuild").join("defines");

        if defines.is_file() {
            let Defines {
                name,
                ver,
                desc,
                deps,
                build_deps,
                pkgbreak,
                pkgrecom,
                provides: pkgprov,
                section,
            } = parse_defines(defines, &mut context, &n, &mut ver)?;

            res.push(Package {
                name: name.ok_or_eyre(format!("Failed to get pkg name: {}", n))?,
                version: ver,
                desc: desc.ok_or_eyre("Failed to get pkg desc")?,
                path,
                pkgbreak,
                pkgrecom,
                provides: pkgprov,
                deps,
                build_deps,
                section,
            })
        } else {
            let verc = ver.clone();
            for j in WalkDir::new(i.path()).min_depth(1).max_depth(1) {
                let mut ver = verc.clone();
                let j = j?;
                if !j.path().is_dir() {
                    continue;
                }

                let defines = j.path().join("defines");
                if defines.is_file() {
                    let Defines {
                        name,
                        ver,
                        desc,
                        deps,
                        build_deps,
                        pkgbreak,
                        pkgrecom,
                        provides: pkgprov,
                        section,
                    } = parse_defines(defines, &mut context, &n, &mut ver)?;

                    res.push(Package {
                        name: name.ok_or_eyre(format!("Failed to get pkg name: {}", n))?,
                        version: ver,
                        desc: desc.ok_or_eyre("Failed to get pkg desc")?,
                        path: path.clone(),
                        pkgbreak,
                        pkgrecom,
                        provides: pkgprov,
                        deps,
                        build_deps,
                        section,
                    });
                }
            }
        }
    }

    Ok(res)
}

struct Defines {
    name: Option<String>,
    ver: String,
    section: String,
    desc: Option<String>,
    deps: Vec<String>,
    build_deps: Vec<String>,
    pkgbreak: Vec<PkgStmt>,
    pkgrecom: Vec<String>,
    provides: Vec<String>,
}

fn parse_defines(
    defines: PathBuf,
    context: &mut HashMap<String, String>,
    n: &str,
    ver: &mut String,
) -> Result<Defines> {
    let mut name = None;
    let mut desc = None;
    let mut pkgsec = None;
    let mut deps = vec![];
    let mut build_deps = vec![];
    let mut pkgbreak = vec![];
    let mut pkgrecom = vec![];
    let mut provides = vec![];

    let c = std::fs::read_to_string(defines)?;
    parse_abbs_file_apml(&c, context).unwrap_or_else(|e| {
        debug!("Failed to parse pkg {n} file using apml: {e}");
        more_parse(&c, context)
    });

    if let Some(v) = context.get("PKGNAME") {
        name = Some(v.replace('=', "").trim().to_string());
    }

    if let Some(v) = context.get("PKGEPOCH") {
        ver.insert(0, ':');
        ver.insert_str(0, v.replace('=', "").trim());
    }

    if let Some(v) = context.get("PKGDES") {
        desc = Some(v.replace('=', "").trim().to_string());
    }

    if let Some(v) = context.get("PKGDEP") {
        for i in v.trim().split_ascii_whitespace() {
            if i == "\\" {
                continue;
            }
            deps.push(i.replace('=', ""));
        }
    }

    if let Some(v) = context.get("BUILDDEP") {
        for i in v.trim().split_ascii_whitespace() {
            if i == "\\" {
                continue;
            }
            build_deps.push(i.replace('=', ""));
        }
    }

    if let Some(v) = context.get("PKGBREAK") {
        for i in v.trim().split_ascii_whitespace() {
            if i == "\\" {
                continue;
            }
            pkgbreak.push(PkgStmt::from(i));
        }
    }

    if let Some(v) = context.get("PKGRECOM") {
        for i in v.trim().split_ascii_whitespace() {
            if i == "\\" {
                continue;
            }
            pkgrecom.push(i.replace('=', ""));
        }
    }

    if let Some(v) = context.get("PKGPROV") {
        for i in v.trim().split_ascii_whitespace() {
            if i == "\\" {
                continue;
            }
            provides.push(i.to_string());
        }
    }

    if let Some(v) = context.get("PKGSEC") {
        pkgsec = Some(v.trim().replace('=', ""));
    }

    Ok(Defines {
        name,
        ver: ver.to_string(),
        desc,
        deps,
        build_deps,
        pkgbreak,
        pkgrecom,
        provides,
        section: pkgsec.unwrap_or("".to_string()),
    })
}

impl From<&str> for PkgStmt {
    fn from(value: &str) -> Self {
        let value = if &value[..1] == "=" {
            &value[1..]
        } else {
            value
        };

        let mut name = String::new();
        let mut comp = String::new();
        let mut version = String::new();
        let mut chars_ed = false;

        let chars = value.chars();
        for c in chars {
            if !is_comp_symbol(&c) && !comp.is_empty() {
                chars_ed = true;
            }

            if !is_comp_symbol(&c) && !chars_ed {
                name.push(c);
            } else if chars_ed {
                version.push(c);
            }

            if is_comp_symbol(&c) {
                comp.push(c);
            }
        }

        PkgStmt {
            name,
            comp,
            version,
        }
    }
}

fn parse_abbs_file_apml(c: &str, context: &mut HashMap<String, String>) -> Result<()> {
    abbs_meta_apml::parse(c, context).map_err(|e| {
        eyre!(e
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("; "))
    })
}

fn more_parse(c: &str, context: &mut HashMap<String, String>) {
    for i in [
        "VER", "REL", "PKGNAME", "PKGEPOCH", "PKGDES", "PKGBREAK", "PKGRECOM", "PKGPROV",
    ] {
        for j in c.split('\n') {
            if let Some(v) = j.strip_prefix(i) {
                context.insert(i.to_string(), v.replace('"', ""));
            }
        }
    }
}

fn is_comp_symbol(c: &char) -> bool {
    ['>', '<', '='].contains(c)
}
