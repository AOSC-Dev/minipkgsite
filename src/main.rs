mod abbs;
use std::{path::PathBuf, sync::Arc, time::Duration};

use abbs::Abbs;
use axum::{
    extract::{Query, State},
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex, time::sleep};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let abbs_url = std::env::var("ABBS_TREE")?;
    let redis = std::env::var("REDIS")?;
    let listen = std::env::var("MINIPKGSITE")?;

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

    let client = redis::Client::open(redis)?;
    let conn = client.get_multiplexed_tokio_connection().await?;
    let abbs = Arc::new(Mutex::new(Abbs::new(conn)?));
    let ac = abbs.clone();

    tokio::spawn(async move {
        loop {
            if let Err(e) = update_db(abbs.clone(), &abbs_url).await {
                error!("{e}");
            }

            sleep(Duration::from_secs(60)).await;
        }
    });

    info!("minipkgsite running at: {}", listen);
    let app = Router::new()
        .route("/package", get(package))
        .route("/all", get(package_all))
        .with_state(ac)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        );

    let listener = tokio::net::TcpListener::bind(&listen).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    name: String,
}

async fn package(
    State(state): State<Arc<Mutex<Abbs>>>,
    Query(payload): Query<Response>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut abbs = state.lock().await;
    let pkg = payload.name;
    let pkg = abbs.get(&pkg).await;

    match pkg {
        Ok(pkg) => Ok(Json(pkg)),
        Err(e) => {
            error!("{e}");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn package_all(
    State(state): State<Arc<Mutex<Abbs>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut abbs = state.lock().await;
    let pkg = abbs.all().await;

    match pkg {
        Ok(pkg) => Ok(Json(pkg)),
        Err(e) => {
            error!("{e}");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn update_db(abbs: Arc<Mutex<Abbs>>, abbs_url: &str) -> Result<()> {
    let mut abbs = abbs.lock().await;
    abbs.update_all(PathBuf::from(abbs_url)).await?;

    Ok(())
}
