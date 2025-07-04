use actix_multipart::{Field, Multipart};
use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError}, get, http::header::ContentType, post, web, HttpResponse};
use anyhow::Context;
use diesel::{r2d2::ConnectionManager, Connection, PgConnection};
use futures_util::StreamExt;
use r2d2::Pool;
use serde::{Serialize, Deserialize};

use crate::{import::process_csv, models::{Deputado, Expense}};

// Representa um argumento contendo uma Unidade Federativa, usado na consulta a deputados por UF.
#[derive(Deserialize)]
struct QueryArgs {
    uf: String,
}

// Representa um argumento opcional contendo a página; utilizado nos endpoints que retornam um
// grande volume de dados e usam paginação para reduzir o consumo de memória e de largura de banda.
#[derive(Deserialize)]
struct PageArgs {
    page: Option<u32>,
}

// Representa a soma das despesas; existe apenas para ser serializado em JSON nos corpos das
// respostas.
#[derive(Serialize)]
struct SumResult {
    soma: f32,
}

// Retorna a soma de todas as despesas.
#[get("/despesas/soma")]
pub async fn soma_todas_as_despesas(
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
        let result = web::block(move || {
            let connection = &mut pool.get().with_context(|| "database error")?;

            Ok(
                Expense::sum_all(connection)?
            )
        })
        .await?
        .map_err(ErrorInternalServerError::<anyhow::Error>)?;

        Ok(
            HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(
                &SumResult { soma: result }
            )?)
        )
}

// Retorna a soma das despesas do deputado com um dado CPF. 
#[get("/despesas/cpf/{cpf}/soma")]
pub async fn soma_despesas(
    cpf: web::Path<String>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
        let cpf = cpf.into_inner();

        let result = web::block(move || {
            let connection = &mut pool.get().with_context(|| "database error")?;

            Ok(
                Expense::sum_all_by_cpf(connection, &cpf)?
            )
        })
        .await?
        .map_err(ErrorInternalServerError::<anyhow::Error>)?;

        Ok(
            HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(
                &SumResult { soma: result }
            )?)
        )
}

// Retorna a lista das despesas de uma determinada unidade federativa.
#[get("/despesas/uf/{uf}")]
pub async fn lista_despesas_por_uf(
    uf: web::Path<String>,
    page: web::Query<PageArgs>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
        let uf = uf.into_inner().trim().to_uppercase();
        let page = if let Some(page) = page.page {
            page
        } else {
            1
        };

        let result = web::block(move || {
            let connection = &mut pool.get().with_context(|| "database error")?;

            Ok(
                Expense::get_expenses_by_uf(connection, &uf, page).with_context(|| "database error")?
            )
        })
        .await?
        .map_err(ErrorInternalServerError::<anyhow::Error>)?;

        Ok(
            HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&result)?)
        )
}

// Retorna a lista das despesas do deputado com um dado CPF.
#[get("/despesas/cpf/{cpf}")]
pub async fn lista_despesas_por_cpf(
    cpf: web::Path<String>,
    page: web::Query<PageArgs>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
        let cpf = cpf.into_inner();
        let page = if let Some(page) = page.page {
            page
        } else {
            1
        };

        let result = web::block(move || {
            let connection = &mut pool.get().with_context(|| "database error")?;

            Ok(
                Expense::get_expenses_by_cpf(connection, &cpf, page).with_context(|| "database error")?
            )
        })
        .await?
        .map_err(ErrorInternalServerError::<anyhow::Error>)?;

        Ok(
            HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&result)?)
        )
}

// Lista os deputados de uma determinada unidade federativa; esta é informada através de um
// parâmetro de URL.
#[get("/deputados")]
pub async fn lista_deputados_por_uf(
    args: web::Query<QueryArgs>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
        let uf = args.uf.trim().to_uppercase();
        if args.uf.len() != 2 {
            return Err(ErrorBadRequest("UF inválida."));
        }
        let result = web::block(move || {
            let mut connection = pool
                .get()
                .with_context(|| "database error")?;

                Deputado::get_all_by_uf(&mut connection, &uf).with_context(|| "database error")
        })
        .await?
        .map_err(ErrorInternalServerError)?;


        Ok(
            HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&result)?)
        )
}

// Processa um CSV contendo as despesas dos deputados; aceita o arquivo como um Multipart, e, caso
// haja múltiplos arquivos, considera apenas o primeiro.
#[post("/processar-ceap")]
pub async fn import_csv(
    mut payload: Multipart,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, actix_web::Error> {
    
    let field = if let Some(field) = payload.next().await {
        field
    } else {
        return Err(ErrorBadRequest("multipart file not found"))
    };

    let field = match field {
        Ok(field) => field,
        Err(e) => return Err(actix_web::error::ErrorBadRequest(e.to_string())),
    };

    let bytes = process_multipart(field).await?;

    web::block(move || -> Result<(), anyhow::Error>{
        let mut connection = pool.get().expect("");
        
        connection.transaction(
            |connection| process_csv(connection, bytes.as_slice())
        )
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("File saved successfully"))
}

// Processa o Multipart contido no corpo de uma requisição
async fn process_multipart(mut field: Field) -> Result<Vec<u8>, actix_web::Error> {
    let mut bytes: Vec<u8> = Vec::new();

        // Write the file content to the file
    while let Some(chunk) = field.next().await {
        match chunk {
            Ok(chunk) => {
                bytes.append(&mut chunk.to_vec());
            },
            Err(e) => return Err(actix_web::error::ErrorBadRequest(e.to_string())),
        };
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::CustomizeConnection;
    use actix_multipart_test::MultiPartFormDataBuilder;
    use actix_web::{test, App};
    use anyhow::Context;
    use diesel::{r2d2::ConnectionManager, Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
    use crate::{get_url_from_env, models::Deputado, routes::import_csv, schema};

    #[derive(Debug)]
    pub struct TransactionCustomizer;

    impl<C: Connection, E> CustomizeConnection<C, E> for TransactionCustomizer {
        fn on_acquire(&self, conn: &mut C) -> Result<(), E> {
            conn.begin_test_transaction().unwrap();
            Ok(())
        }
    }

    pub fn build_test_connection_pool() -> anyhow::Result<Pool<ConnectionManager<PgConnection>>>{
        let url = get_url_from_env()?;

        let manager = ConnectionManager::<PgConnection>::new(url);
        
        Ok(
            Pool::builder()
                .test_on_check_out(true)
                .connection_customizer(Box::new(TransactionCustomizer))
                .build(manager)
                .with_context(|| "failed to build test connection pool")?
        )
    }

    fn get_csv() -> &'static str {
        "txNomeParlamentar;cpf;ideCadastro;nuCarteiraParlamentar;nuLegislatura;sgUF;sgPartido;codLegislatura;numSubCota;txtDescricao;numEspecificacaoSubCota;txtDescricaoEspecificacao;txtFornecedor;txtCNPJCPF;txtNumero;indTipoDocumento;datEmissao;vlrDocumento;vlrGlosa;vlrLiquido;numMes;numAno;numParcela;txtPassageiro;txtTrecho;numLote;numRessarcimento;datPagamentoRestituicao;vlrRestituicao;nuDeputadoId;ideDocumento;urlDocumento
Ninguém;;;;2023;NA;;57;1;Descrição;0;;Fornecedor;CNPJ-fornecedor;1984;0;2025-02-07T00:00:00;1467;0;1467;1;2025;0;;;0;;;;0;0;https://test.url/0000.pdf
Jorge;22488012033;;;2023;PB;;57;1;Descrição;0;;Fornecedor;CNPJ-fornecedor;1984;0;2025-02-07T00:00:00;1467;0;1467;2;2025;0;;;0;;;;0;0;https://test.url/0001.pdf
Zé;71838787089;;;2023;RJ;;57;1;Descrição;0;;Fornecedor;CNPJ-fornecedor;1984;0;2025-02-07T00:00:00;1467;0;1467;3;2025;0;;;0;;;;0;0;https://test.url/0002.pdf
Jorge;22488012033;;;2023;PB;;57;1;Descrição;0;;Fornecedor;CNPJ-fornecedor;1984;0;2025-02-07T00:00:00;1467;0;1467;2;2025;0;;;0;;;;0;0;https://test.url/0001.pdf
Jorge;22488012033;;;2023;PB;;57;1;Descrição;0;;Fornecedor;CNPJ-fornecedor;1984;0;;1467;0;1467;2;2025;0;;;0;;;;0;0;https://test.url/0001.pdf"
    }

    #[actix_web::test]
    async fn process_request_with_wrong_method() {
        let pool = build_test_connection_pool().unwrap();
        let app = test::init_service(
            App::new()
                .service(import_csv)
                .app_data(web::Data::new(pool.clone()))
        ).await;

        let req =
            test::TestRequest::get()
            .uri("/processar-ceap")
            .set_payload(get_csv())
            .to_request();

        let response = test::call_service(&app, req).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn process_request_with_valid_payload() {
        use crate::schema::deputados::dsl::deputados;

        let pool = build_test_connection_pool().unwrap();
        let app = test::init_service(
            App::new()
                .service(import_csv)
                .app_data(web::Data::new(pool.clone()))
        ).await;

        let (header, payload): ((String, String), Vec<u8>) = MultiPartFormDataBuilder::new().with_text("file", get_csv()).build();

        let req =
            test::TestRequest::post()
            .uri("/processar-ceap")
            .insert_header(header)
            .set_payload(payload)
            .to_request();

        let response = test::call_service(&app, req).await;
        
        assert!(response.status().is_success());
        
        let mut connection = pool.get().unwrap();

        let dep1 = deputados
            .filter(schema::deputados::cpf.eq("22488012033"))
            .select(Deputado::as_select())
            .first(&mut connection)
            .unwrap();

        let dep2 = deputados
            .filter(schema::deputados::cpf.eq("71838787089"))
            .select(Deputado::as_select())
            .first(&mut connection)
            .unwrap();

        assert_eq!(dep1.cpf, "22488012033");
        assert_eq!(dep2.cpf, "71838787089");
        assert_eq!(dep1.nome, "Jorge");
        assert_eq!(dep2.nome, "Zé");
    }

    #[actix_web::test]
    async fn process_request_without_payload() {
        let pool = build_test_connection_pool().unwrap();
        let app = test::init_service(
            App::new()
                .service(import_csv)
                .app_data(web::Data::new(pool.clone()))
        ).await;

        let req =
            test::TestRequest::post()
            .uri("/processar-ceap")
            .to_request();

        let response = test::call_service(&app, req).await;
        
        assert!(response.status().is_client_error());
    }

}
