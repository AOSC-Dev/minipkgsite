mod abbs;
use std::path::PathBuf;

use abbs::Abbs;
use eyre::Result;
use tokio::time::sleep;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let abbs_url = std::env::var("ABBS_TREE")?;
    let redis = std::env::var("REDIS")?;

    let env_log = EnvFilter::try_from_default_env();

    if let Ok(filter) = env_log {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .event_format(
                        tracing_subscriber::fmt::format()
                            .with_file(true)
                            .with_line_number(true),
                    )
                    .with_filter(filter),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .event_format(
                        tracing_subscriber::fmt::format()
                            .with_file(true)
                            .with_line_number(true),
                    )
                    .with_filter(LevelFilter::INFO),
            )
            .init();
    }

    loop {
        if let Err(e) = tokio::try_join!(update_db(redis.clone(), abbs_url.clone())) {
            error!("{e}");
        }

        sleep(std::time::Duration::from_secs(60)).await;
    }
}

async fn update_db(redis: String, abbs_url: String) -> Result<()> {
    let abbs = Abbs::new(&redis)?;
    abbs.update_all(PathBuf::from(abbs_url)).await?;

    Ok(())
}
