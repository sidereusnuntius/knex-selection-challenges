# Desafio

# Executando o projeto

## Com Docker

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
