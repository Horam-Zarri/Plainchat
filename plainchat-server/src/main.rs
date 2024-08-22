use std::{env, process};

use axum::response::IntoResponse;
use axum::{http::HeaderValue, response::Html};
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::Router;
use axum::routing::get;
use dotenv::dotenv;
use redis::{aio, AsyncCommands, Client, RedisConnectionInfo};
use redis::aio::MultiplexedConnection;
use socketioxide::layer::SocketIoLayer;
use socketioxide::SocketIo;
use sqlx::{PgPool, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::Any;
use tower_http::{cors::CorsLayer, trace::{Trace, TraceLayer}};
use tracing::{info, trace, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod routes;
mod models;
mod seed;
mod ws;

pub mod util;
mod auth_extractor;

#[derive(Clone, Debug)]
struct AppState {
    db: PgPool,
    redis: MultiplexedConnection
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    check_env();


    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&dotenv::var("DATABASE_URL")?).await?;

    let redis_client = Client::open(dotenv::var("REDIS_URL")?)?;
    let redis = redis_client.get_multiplexed_tokio_connection().await?;

    sqlx::migrate!("./migrations")
        .run(&db)
        .await?;

    for arg in env::args() {
        if arg == "SEED_DB" {
            println!("Seeding Database...");
            seed::populate_db(&db).await;
            println!("Done seeding!");
        }
    }

    let state = AppState {db, redis};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "plainchat_server=trace,tower_http=debug,axum::rejection=debug".into()
            })
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("TRACING INITIALIZED");

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api/user", routes::user::router())
        .nest("/api/group", routes::group::router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new()
                    .allow_origin(Any)
                    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                    .allow_methods(Any))
                //.allow_origin(CorsLayer::permissive())//"http://0.0.0.0:3000".parse::<HeaderValue>().unwrap())
                .layer(ws::layer(state.clone()))
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    "Route is OK"
}
fn check_env() {

    const REQUIRED_VARS: &[&'static str] = &[
        "DATABASE_URL",
        "JWT_SECRET",
        "REDIS_URL"
    ];

    for v in REQUIRED_VARS.iter() {
        if dotenv::var(v).is_err() {
            println!("{v} must be set in .env");
            std::process::exit(1);
        }
    }
}

