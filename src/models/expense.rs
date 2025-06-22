use chrono::NaiveDate;
use diesel::prelude::*;
use serde::Deserialize;

use crate::{models::deputado::Deputado, schema::expenses};

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Deputado))]
#[diesel(table_name = expenses)]
pub struct Expense {
    pub id: i32,
    pub data_emissao: NaiveDate,
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
    pub data_emissao: NaiveDate,
    pub url_documento: Option<String>,
    pub deputado_id: i32,
}

#[derive(Deserialize)]
pub struct ExpenseFromCsv {
    #[serde(rename = "txtFornecedor")]
    pub fornecedor: String,
    #[serde(rename = "vlrLiquido")]
    pub valor_liquido: f32,
    #[serde(rename = "datEmissao")]
    pub data_emissao: Option<String>,
    #[serde(rename = "numMes")]
    pub mes: u32,
    #[serde(rename = "numAno")]
    pub ano: i32,
    #[serde(rename = "urlDocumento")]
    pub url_documento: Option<String>,
}