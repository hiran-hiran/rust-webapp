use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::{postgres::PgConnectOptions, PgPool};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let database_config = DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        username: "app".into(),
        password: "passwd".into(),
        database: "app".into(),
    };
    let connection_pool = connect_database_with(database_config);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .with_state(connection_pool);
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);
    return Ok(axum::serve(listener, app).await?);
}

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl From<DatabaseConfig> for PgConnectOptions {
    fn from(config: DatabaseConfig) -> Self {
        PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.username)
            .password(&config.password)
            .database(&config.database)
    }
}

fn connect_database_with(config: DatabaseConfig) -> PgPool {
    PgPool::connect_lazy_with(config.into())
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[tokio::test]
async fn health_check_works() {
    let response = health_check().await;
    assert_eq!(response, StatusCode::OK);
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    let connection_result = sqlx::query("SELECT 1").fetch_one(&db).await;
    match connection_result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
#[sqlx::test]
async fn health_check_db_works(pool: sqlx::PgPool) {
    let response = health_check_db(State(pool)).await;
    assert_eq!(response, StatusCode::OK);
}
