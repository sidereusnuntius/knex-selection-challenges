# Endpoints

* ```GET /despesas/soma```: retorna a soma de todas as despesas.
* ```GET /despesas/cpf/{cpf}/soma```: retorna a soma das despesas do deputado com o CPF dado.
* ```GET /despesas/uf/{uf}```: lista todas as despesas de uma unidade federativa. Possui um parâmetro opcional, page, que informa a página: /despesas/uf/{uf}?page=2.
* ```GET /despesas/cpf/{cpf}```: lista as despesas do deputado com o CPF informado. Possui um parâmetro opcional, page, que informa a página.
* ```GET /deputados?uf={uf}```: lista deos deputados da unidade federativa dada.
* ```POST /processar-ceap```: processa o CSV enviado no corpo da requisição como um multipart.

# Executando o projeto

## Com Docker

Para fazer o Actix e o Diesel funcionarem no Docker, eu me baseei nos seguintes exemplos: [[1]](https://medium.com/@aniketsuryawanshixz1/building-a-rust-api-with-actix-web-diesel-postgres-and-docker-09b0958552aa) e [[2]](https://www.codefeetime.com/post/docker-config-for-actix-web-diesel-and-postgres/).

```
docker compose up -d db
docker compose run --rm app diesel setup
docker compose run --rm app diesel migration run
docker compose up app
```

## Sem Docker

### Requisitos
* [Rust e Cargo](https://www.rust-lang.org/tools/install)
* [Postgres](https://www.postgresql.org/download/)
* [Diesel CLI](http://diesel.rs/guides/getting-started.html)

### Execução
1. Crie no .env uma variável chamada DATABASE_URL contendo a URL de acesso ao banco de dados.
2. Use o Diesel CLI para criar o banco de dados e executar as migrações.
   ```
   diesel setup
   diesel migration run
   ```
3. Você pode compilar o projeto, ou executá-lo diretamente com o seguinte comando:
   ```
   cargo run --release
   ```
Para executar os testes, use ```cargo test```. Para a documentação, ```cargo doc --no-deps --open```.
