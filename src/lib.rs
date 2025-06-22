use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError}, post, web, HttpResponse};

// pub mod error;
pub mod models;
pub mod schema;
pub mod validate;
pub mod import;

use actix_multipart::{Field, Multipart};
use anyhow::Context;
use diesel::{prelude::*, r2d2::ConnectionManager};
use dotenvy::dotenv;
use r2d2::Pool;
use futures_util::StreamExt;
use std::env;

use crate::import::process_csv;

fn get_url_from_env() -> Result<String, env::VarError> {
    dotenv().ok();

    env::var("DATABASE_URL")
}

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

async fn process_multipart(mut field: Field) -> Result<Vec<u8>, actix_web::Error> {
    let mut bytes: Vec<u8> = Vec::new();

        // Write the file content to the file
    while let Some(chunk) = field.next().await {
        match chunk {
            Ok(chunk) => {
                bytes.append(&mut chunk.to_vec());
            },
            Err(e) => return Err(actix_web::error::ErrorBadRequest(e.to_string())),
        };
    }

    Ok(bytes)
}

#[post("/processar-ceap")]
pub async fn import_csv(
    mut payload: Multipart,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
    
    let field = if let Some(field) = payload.next().await {
        field
    } else {
        return Err(ErrorBadRequest("multipart file not found"))
    };

    let field = match field {
        Ok(field) => field,
        Err(e) => return Err(actix_web::error::ErrorBadRequest(e.to_string())),
    };

    let bytes = process_multipart(field).await?;

    web::block(move || -> Result<(), anyhow::Error>{
        let mut connection = pool.get().expect("");

        connection.transaction(
            |connection| process_csv(connection, bytes.as_slice())
        )
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("File saved successfully"))
}