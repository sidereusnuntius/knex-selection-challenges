use diesel::prelude::*;
use serde::{Serialize,Deserialize};

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = crate::schema::deputados)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Deputado {
    #[serde(skip)]
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

impl Deputado {
    pub fn get_all_by_uf(connection: &mut PgConnection, uf_busca: &str) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::deputados::dsl::*;

        deputados
            .filter(uf.eq(uf_busca))
            .select(Deputado::as_select())
            .load(connection)
    }
}