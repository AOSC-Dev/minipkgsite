use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use eyre::{eyre, Context, OptionExt, Result};
use redis::{AsyncCommands, Client};
use serde::Serialize;
use tracing::{info, warn};
use walkdir::WalkDir;

pub struct Abbs {
    client: Client,
    pkgs: Arc<DashMap<String, Package>>,
}

#[derive(Debug, Serialize)]
struct Package {
    name: String,
    version: String,
    desc: String,
    path: String,
    deps: Vec<String>,
    build_deps: Vec<String>,
    pkgbreak: Vec<PkgStmt>,
    pkgrecom: Vec<String>,
    provides: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PkgStmt {
    name: String,
    comp: String,
    version: String,
}

// impl ToRedisArgs for Package {
//     fn write_redis_args<W>(&self, out: &mut W)
//     where
//         W: ?Sized + redis::RedisWrite {
//         self.name.write_redis_args(out)?;
//     }
// }

impl Abbs {
    pub fn new(url: &str) -> Result<Abbs> {
        let client = redis::Client::open(url).context("Failed to connect redis database")?;
        let pkgs = Arc::new(DashMap::new());

        Ok(Abbs { client, pkgs })
    }

    pub async fn update_all(&self, git_path: PathBuf) -> Result<()> {
        let pkg = self.pkgs.clone();
        let res =
            tokio::task::spawn_blocking(move || collection_packages(git_path, &pkg)).await??;

        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        for i in res {
            conn.set(&i.name, serde_json::to_string(&i)?).await?;
        }

        Ok(())
    }
}

fn collection_packages(git_path: PathBuf, pkgs: &DashMap<String, Package>) -> Result<Vec<Package>> {
    let dir = WalkDir::new(git_path).max_depth(2).min_depth(2);
    let mut res = vec![];

    for i in dir {
        let mut ver = String::new();
        let mut name = None;
        let mut desc = None;
        let i = i?;

        let p = i.path().display().to_string();

        if p.contains(".git") || p.contains("groups") {
            continue;
        }

        let mut context = HashMap::new();
        let mut pkgbreak = vec![];
        let mut pkgrecom = vec![];
        let mut pkgprov = vec![];
        let path = i
            .path()
            .parent()
            .ok_or_eyre("Parent does not exist")?
            .display()
            .to_string();
        let mut deps = vec![];
        let mut build_deps = vec![];

        let n = i.file_name().to_string_lossy();

        info!("scanning {}", n);

        let spec = i.path().join("spec");

        if spec.is_file() {
            let c = std::fs::read_to_string(spec)?;
            parse_abbs_file_apml(&c, &mut context).unwrap_or_else(|e| {
                warn!("{e}");
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
            let c = std::fs::read_to_string(defines)?;
            parse_abbs_file_apml(&c, &mut context).unwrap_or_else(|e| {
                warn!("{e}");
                more_parse(&c, &mut context)
            });

            if let Some(v) = context.get("PKGNAME") {
                name = Some(v.replace("=", ""));
            }

            if let Some(v) = context.get("PKGEPOCH") {
                ver.insert(0, ':');
                ver.insert_str(0, &v.replace("=", ""));
            }

            if let Some(v) = context.get("PKGDES") {
                desc = Some(v.to_string());
            }

            if let Some(v) = context.get("PKGDEP") {
                for i in v.split_ascii_whitespace() {
                    deps.push(i.to_string());
                }
            }

            if let Some(v) = context.get("BUILDDEP") {
                for i in v.split_ascii_whitespace() {
                    build_deps.push(i.to_string());
                }
            }

            if let Some(v) = context.get("PKGBREAK") {
                for i in v.split_ascii_whitespace() {
                    pkgbreak.push(PkgStmt::from(i));
                }
            }

            if let Some(v) = context.get("PKGRECOM") {
                for i in v.split_ascii_whitespace() {
                    pkgrecom.push(i.replace("=", ""));
                }
            }

            if let Some(v) = context.get("PKGPROV") {
                for i in v.split_ascii_whitespace() {
                    pkgprov.push(i.to_string());
                }
            }
        } else {
            warn!("{} has no autobuild dir", i.path().display());
            continue;
        }

        res.push(Package {
            name: name.ok_or_eyre("Failed to get pkg name")?,
            version: ver,
            desc: desc.ok_or_eyre("Failed to get pkg desc")?,
            path,
            pkgbreak,
            pkgrecom,
            provides: pkgprov,
            deps,
            build_deps,
        })
    }

    Ok(res)
}

impl From<&str> for PkgStmt {
    fn from(value: &str) -> Self {
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
            .join(";"))
    })
}

fn more_parse(c: &str, context: &mut HashMap<String, String>) {
    for i in [
        "VER", "REL", "PKGNAME", "PKGEPOCH", "PKGDES", "PKGBREAK", "PKGRECOM", "PKGPROV",
    ] {
        for j in c.split('\n') {
            if let Some(v) = j.strip_prefix(i) {
                context.insert(i.to_string(), v.replace("\"", ""));
            }
        }
    }
}

fn is_comp_symbol(c: &char) -> bool {
    ['>', '<', '='].contains(c)
}
