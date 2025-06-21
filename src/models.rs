use chrono::NaiveDate;
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

use crate::schema::{deputados, expenses};

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

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Deputado))]
#[diesel(table_name = expenses)]
pub struct Expense {
    pub id: i32,
    pub data_despesa: NaiveDate,
    pub fornecedor: String,
    pub valor_liquido: f32,
    pub url_documento: Option<String>,
    pub deputado_id: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = expenses)]
pub struct NewExpense {
    pub fornecedor: String,
    pub valor_liquido: f32,
    pub data_despesa: NaiveDate,
    pub url_documento: Option<String>,
    pub deputado_id: i32,
}

#[derive(Deserialize)]
pub struct ExpenseFromCsv {
    #[serde(rename = "txtFornecedor")]
    pub fornecedor: String,
    #[serde(rename = "vlrLiquido")]
    pub valor_liquido: f32,
    #[serde(rename = "numMes")]
    pub mes: u32,
    #[serde(rename = "numAno")]
    pub ano: i32,
    #[serde(rename = "urlDocumento")]
    pub url_documento: Option<String>,
}