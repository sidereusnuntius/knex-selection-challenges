use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::{models::deputado::Deputado, schema::expenses};

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Deputado))]
#[diesel(table_name = expenses)]
pub struct Expense {
    pub id: i32,
    pub data_emissao: NaiveDateTime,
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
    pub data_emissao: NaiveDateTime,
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

diesel::table! {
    despesa_com_deputado (expense_id) {
        expense_id -> Int4,
        data_emissao -> Nullable<Timestamp>,
        fornecedor -> Varchar,
        valor_liquido -> Float4,
        url_documento -> Nullable<Varchar>,
        nome -> Varchar,
        cpf -> Varchar,
    }
}

#[derive(Debug, Queryable, Selectable, Serialize, Identifiable, Associations, PartialEq)]
#[diesel(belongs_to(Expense))]
#[diesel(primary_key(expense_id))]
#[diesel(table_name = despesa_com_deputado)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DespesaComDeputado {
    pub expense_id: i32,
    pub data_emissao: Option<NaiveDateTime>,
    pub fornecedor: String,
    pub valor_liquido: f32,
    pub url_documento: Option<String>,
    pub nome: String,
    pub cpf: String,
}

impl Expense {
    pub fn get_expenses_by_cpf(connection: &mut PgConnection, cpf_busca: &str, mut page: u32) -> Result<Vec<DespesaComDeputado>, Error> {
        use self::despesa_com_deputado::dsl::*;
        if page == 0 { page = 1; }

        Ok(
            despesa_com_deputado
            .filter(cpf.eq(cpf_busca))
            .select(DespesaComDeputado::as_select())
            .limit(20)
            .offset(20 * (page as i64 - 1))
            .load(connection)?
        )
    }
}