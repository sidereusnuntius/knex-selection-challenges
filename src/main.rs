use std::{io, process::exit};

use actix_web::{web, App, HttpServer};
use knex_selection_challenges::{build_connection_pool, routes::import_csv};

#[actix_web::main]
async fn main() -> io::Result<()>{

    let pool = match build_connection_pool() {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("{e:?}");
            exit(1);
        },
    };
    
    // let file_name = env::var("FILE").unwrap();

    // let file = File::open(&file_name).unwrap();

    HttpServer::new(move || {
        App::new()
        .service(import_csv)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
