use std::io;
use std::collections::HashMap;

use anyhow::Error;
use anyhow::{bail, Context};
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::{Deputado, ExpenseFromCsv, NewExpense};
use crate::models::NovoDeputado;
use crate::schema;
use crate::validate::valida_cpf;

pub fn process_csv<T>(connection: &mut PgConnection, reader: T) -> Result<(), Error>
where
    T: io::Read
    {
    let mut cache: HashMap<String, i32> = HashMap::new();
    let mut rdr =
    csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .double_quote(false)
        .from_reader(reader);

    let headers = rdr
        .headers()
        .with_context(|| "failed to parse CSV headers")?
        .clone();

    let mut expenses = Vec::new();

    for record in rdr.records() {
        let record = match record {
            Ok(r) => r,
            Err(e) => {
                if let Some(p) = e.position() {
                    bail!("failed to parse CSV at line {}", p.line());
                } else {
                    bail!("failed to parse CSV.");
                }
            },
        };
        if let Some("NA") = record.get(5) {
            continue;
        }
        // if record.get(5).unwrap() == "NA" { continue; }

        let dep_cpf = if let Some(cpf) = record.get(1) {
            cpf
        } else {
            continue;
        };
        
        let current_id = if let Some(id) = cache.get(dep_cpf) {
            *id
        } else if let Ok(id) = get_id_by_cpf(connection, dep_cpf) {
                cache.insert(dep_cpf.to_string(), id);
                println!("CPF {dep_cpf} has already been registered with id {id}.");
                id
        } else {
            let r: NovoDeputado = record
                .deserialize(Some(&headers))
                .with_context(|| "failed to deserialize a record.")?;

            let result = insert_deputado(connection, r)?;
            
            cache.insert(result.cpf.clone(), result.id);
            println!("Registered: {:?}.", result);
            result.id
        };

        let expense: ExpenseFromCsv = record
            .deserialize(Some(&headers))
            .with_context(|| "failed to deserialize expense.")?;

        expenses.push(NewExpense {
            data_despesa: NaiveDate::from_ymd_opt(expense.ano, expense.mes, 1).unwrap(),
            deputado_id: current_id,
            fornecedor: expense.fornecedor,
            valor_liquido: expense.valor_liquido,
            url_documento: expense.url_documento,
        });
        // println!("Insert: {:?}", expense);
        if expenses.len() == 10000 {
            diesel::insert_into(schema::expenses::table)
            .values(&expenses)
            // .returning(Expense::as_returning())
            .execute(connection)
            .with_context(|| "batch insertion failed.")?;
        expenses.clear();
        }
    }
    if expenses.len() > 0 {
        diesel::insert_into(schema::expenses::table)
                .values(&expenses)
                // .returning(Expense::as_returning())
                .execute(connection)
                .with_context(|| "batch insertion failed.")?;
    }
    Ok(())
}

fn get_id_by_cpf(connection: &mut PgConnection, cpf: &str) -> Result<i32, Error> {
    use self::schema::deputados::dsl::deputados;

    Ok(
        deputados
            .filter(schema::deputados::cpf.eq(cpf))
            .select(schema::deputados::id)
            .first(connection)?
    )
}

fn insert_deputado(connection: &mut PgConnection, deputado: NovoDeputado) -> Result<Deputado, Error> {
    if !valida_cpf(&deputado.cpf) {
        bail!("invalid CPF.");
    }
    
    Ok(
        diesel::insert_into(schema::deputados::table)
            .values(deputado)
            .returning(Deputado::as_returning())
            .get_result(connection)
            .with_context(|| "failed to insert record.")?
    )
}

#[cfg(test)]
mod tests {
    use std::env;
    use dotenvy::dotenv;

    fn get_csv() -> &'static str {
        r#""txNomeParlamentar";"cpf";"ideCadastro";"nuCarteiraParlamentar";"nuLegislatura";"sgUF";"sgPartido";"codLegislatura";"numSubCota";"txtDescricao";"numEspecificacaoSubCota";"txtDescricaoEspecificacao";"txtFornecedor";"txtCNPJCPF";"txtNumero";"indTipoDocumento";"datEmissao";"vlrDocumento";"vlrGlosa";"vlrLiquido";"numMes";"numAno";"numParcela";"txtPassageiro";"txtTrecho";"numLote";"numRessarcimento";"datPagamentoRestituicao";"vlrRestituicao";"nuDeputadoId";"ideDocumento";"urlDocumento"
        Ninguém;"";"";"";"2023";"NA";"";"57";"1";"Descrição";"0";"";"Fornecedor";"CNPJ-fornecedor";"1984";"0";"2025-02-07T00:00:00";"1467";"0";"1467";"1";"2025";"0";"";"";"0";"";"";"";"0";"0";"https://test.url/0000.pdf"
        Jorge;22488012033;"";"";"2023";"PB";"";"57";"1";"Descrição";"0";"";"Fornecedor";"CNPJ-fornecedor";"1984";"0";"2025-02-07T00:00:00";"1467";"0";"1467";"2";"2025";"0";"";"";"0";"";"";"";"0";"0";"https://test.url/0001.pdf"
        Zé;71838787089;"";"";"2023";"RJ";"";"57";"1";"Descrição";"0";"";"Fornecedor";"CNPJ-fornecedor";"1984";"0";"2025-02-07T00:00:00";"1467";"0";"1467";"3";"2025";"0";"";"";"0";"";"";"";"0";"0";"https://test.url/0002.pdf"
        Jorge;22488012033;"";"";"2023";"PB";"";"57";"1";"Descrição";"0";"";"Fornecedor";"CNPJ-fornecedor";"1984";"0";"2025-02-07T00:00:00";"1467";"0";"1467";"2";"2025";"0";"";"";"0";"";"";"";"0";"0";"https://test.url/0001.pdf""#
    }

    use super::*;

    fn get_connection() -> PgConnection{
        dotenv().ok();

        let db_url = env::var("DATABASE_URL").unwrap();
        PgConnection::establish(&db_url).unwrap()
    }

    #[test]
    fn processa_csv_valido() {
        use self::schema::deputados::dsl::deputados;
        use self::schema::expenses::dsl::expenses;

        let connection = &mut get_connection();
        
        connection.test_transaction(|connection| {
            assert!(process_csv(connection, get_csv().as_bytes()).is_ok());

            assert_eq!(deputados
                // .select(schema::deputados::id)
                .count()
                .get_result(connection), Ok(2));

                assert_eq!(expenses
                    // .select(schema::deputados::id)
                    .count()
                    .get_result(connection), Ok(3));

            Ok::<(), Error>(())
        });
    }

    fn deputado_com_cpf(cpf: String) -> NovoDeputado {
        NovoDeputado {
            nome: "Teste".to_string(),
            uf: "PB".to_string(),
            cpf: cpf,
            partido: Some("Partido Pirata".to_string()),
        }
    }

    #[test]
    fn rejeita_deputado_com_cpf_invalido() {
        let connection = &mut get_connection();

        connection.test_transaction(|connection| {
            assert!(insert_deputado(connection, deputado_com_cpf(String::from(""))).is_err());
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("123456789"))).is_err());
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("abc"))).is_err());
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("9755404260035"))).is_err());

            Ok::<(), Error>(())
        })
    }

    #[test]
    fn aceita_deputado_com_cpf_valido() {
        let connection = &mut get_connection();

        connection.test_transaction(|connection| {
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("2673718605"))).is_ok());
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("88293388005"))).is_ok());
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("70042234000"))).is_ok());
            assert!(insert_deputado(connection, deputado_com_cpf(String::from("700422340"))).is_ok());

            Ok::<(), Error>(())
        })
    }
}