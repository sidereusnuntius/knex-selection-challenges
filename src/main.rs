use std::fs::File;

use diesel::{prelude::*, result::Error};
use dotenvy::dotenv;
use knex_selection_challenges::process_csv;
use std::env;

fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    let file_name = env::var("FILE").unwrap();
    let conn = &mut PgConnection::establish(&db_url)
        .unwrap();
    let file = File::open(&file_name).unwrap();

    match conn.transaction(|connection| {
        process_csv(connection, file)
    }) {
        Ok(_) => println!("Insert succeeded!"),
        Err(e) => println!("Error: {e:?}"),
    };
}
