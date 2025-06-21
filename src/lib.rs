use std::io::Read;
use std::{collections::HashMap, fs::File};

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::PgConnection;

use crate::models::{Deputado, Expense, ExpenseFromCsv, NewExpense};
use crate::models::NovoDeputado;

mod models;
mod schema;

pub fn process_csv(connection: &mut PgConnection, file: File) -> Result<(), Error>{
    let mut cache: HashMap<String, i32> = HashMap::new();
    let mut rdr =
    csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .double_quote(false)
        .from_reader(file);

    let headers = rdr.headers().unwrap().clone();

    let mut expenses = Vec::new();

    for record in rdr.records() {
        let record = record.unwrap();
        if record.get(5).unwrap() == "NA" { continue; }

        let dep_cpf = record.get(1).unwrap();
        
        let current_id = if let Some(id) = cache.get(dep_cpf) {
            *id
        } else if let Ok(id) = get_id_by_cpf(connection, dep_cpf) {
                cache.insert(dep_cpf.to_string(), id);
                println!("CPF {dep_cpf} has already been registered with id {id}.");
                id
        } else {
            let r: NovoDeputado = record.deserialize(Some(&headers)).unwrap();

            let result = insert_deputado(connection, r).unwrap();
            
            cache.insert(result.cpf.clone(), result.id);
            println!("Registered: {:?}.", result);
            result.id
        };

        let expense: ExpenseFromCsv = record.deserialize(Some(&headers)).unwrap();
        expenses.push(NewExpense {
            data_despesa: NaiveDate::from_ymd_opt(expense.ano, expense.mes, 1).unwrap(),
            deputado_id: current_id,
            fornecedor: expense.fornecedor,
            valor_liquido: expense.valor_liquido,
            url_documento: expense.url_documento,
        });
        // println!("Insert: {:?}", expense);
        if expenses.len() == 2000 {
            diesel::insert_into(schema::expenses::table)
            .values(&expenses)
            // .returning(Expense::as_returning())
            .execute(connection)?;
        expenses.clear();
        }
    }
    if expenses.len() > 0 {
        diesel::insert_into(schema::expenses::table)
                .values(&expenses)
                // .returning(Expense::as_returning())
                .execute(connection)?;
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

fn insert_deputado(connection: &mut PgConnection, deputado: NovoDeputado) -> Result<Deputado, diesel::result::Error> {
    
    Ok(
        diesel::insert_into(schema::deputados::table)
            .values(deputado)
            .returning(Deputado::as_returning())
            .get_result(connection)?
    )
}

fn valida_cpf(cpf: &str) -> bool {
    if cpf.chars().filter(|c| c.is_numeric()).count() != cpf.len() { return false; }
    
    // Meio porco, mas funciona.
    let cpf = match cpf.len() {
        9 => cpf.to_string() + &"00",
        10 => cpf.to_string() + &"0",
        11 => cpf.to_string(),
        _ => return false,
    };

    let mut cpf_digits = Vec::with_capacity(11);

    cpf.bytes().for_each(|c| cpf_digits.push((c - b'0') as u16));
    let valida_1o_digito = cpf_digits[..9]
        .iter()
        .enumerate()
        .map(
            |(i, digit)| {
                println!("{digit} * {}", 10-i);
                digit * (10 - i as u16)}
        ).sum::<u16>();

    let valida_2o_digito = cpf_digits[..=9]
        .iter()
        .enumerate()
        .map(
            |(i, digit)| {
                println!("{digit} * {}", 11-i);
                digit * (11 - i as u16)}
        ).sum::<u16>();
    println!("{}\n{}\n{}\n{}\n{}\n{}",
        valida_1o_digito,
        valida_2o_digito,
        ((valida_1o_digito * 10) % 11) % 10,
        ((valida_2o_digito * 10) % 11) % 10,
        cpf_digits.get(9).unwrap_or(&0),
        cpf_digits.get(10).unwrap_or(&0) 
    );
    if ((valida_1o_digito * 10) % 11) % 10 != *cpf_digits.get(9).unwrap_or(&0) 
        || ((valida_2o_digito * 10) % 11) % 10 != *cpf_digits.get(10).unwrap_or(&0) {
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_cpf_valido_com_11_digitos() {
        assert!(valida_cpf("52998224725"));
    }

    #[test]
    fn aceita_cpf_valido_com_9_digitos() {
        assert!(valida_cpf("770338410"));
    }

    #[test]
    fn rejeita_cpf_com_tamanho_invalido() {
        assert_eq!(valida_cpf(""), false);
        assert_eq!(valida_cpf("12"), false);
        assert_eq!(valida_cpf("7703384"), false);
    }

    #[test]
    fn rejeita_cpf_invalido() {
        assert_eq!(valida_cpf("12345678900"), false);
    }
}