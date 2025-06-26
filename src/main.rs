use std::{io, process::exit};

use actix_web::{web, App, HttpServer};
use knex_selection_challenges::{build_connection_pool, routes::{import_csv, lista_deputados_por_uf, lista_despesas_por_cpf, lista_despesas_por_uf, soma_despesas}};

#[actix_web::main]
async fn main() -> io::Result<()>{
    colog::init();
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
        .service(lista_deputados_por_uf)
        .service(lista_despesas_por_cpf)
        .service(lista_despesas_por_uf)
        .service(soma_despesas)
        .app_data(web::Data::new(pool.clone()))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
