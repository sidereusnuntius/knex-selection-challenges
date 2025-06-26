// pub mod error;
pub mod models;
pub mod schema;
pub mod validate;
pub mod import;
pub mod routes;

use anyhow::Context;
use diesel::{prelude::*, r2d2::ConnectionManager};
use dotenvy::dotenv;
use r2d2::Pool;
use std::env;

// Retorna o URL de conexão com o banco de dados extraído da variável de ambiente DATABASE_URL.
fn get_url_from_env() -> Result<String, env::VarError> {
    dotenv().ok();

    env::var("DATABASE_URL")
}

// Gera pool de conexões com o banco de dados.
pub fn build_connection_pool() -> anyhow::Result<Pool<ConnectionManager<PgConnection>>>{
    let url = get_url_from_env()?;

    let manager = ConnectionManager::<PgConnection>::new(url);
    
    Ok(
        Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .with_context(|| "failed to build connection pool")?
    )
}
