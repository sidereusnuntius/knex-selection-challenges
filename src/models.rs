use diesel::prelude::*;
use serde::Deserialize;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::deputados)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Deputado {
    pub id: i32,
    pub nome: String,
    pub uf: String,
    pub cpf: String,
    pub partido: Option<String>,
}

use crate::schema::deputados;

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = deputados)]
pub struct NovoDeputado {
    #[serde(rename = "txNomeParlamentar")]
    pub nome: String,
    #[serde(rename = "sgUF")]
    pub uf: String,
    pub cpf: String,
    #[serde(rename = "sgPartido")]
    pub partido: Option<String>,
}