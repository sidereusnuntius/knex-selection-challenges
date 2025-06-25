FROM rust:1.87 as build

RUN apt-get update && \
    apt-get install -y libpq-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app
# RUN cargo init --bin

COPY src ./src
COPY migrations ./migrations
COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libpq5 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/cargo/bin/diesel /usr/local/bin
COPY --from=build /app/target/release/knex_app /usr/local/bin/knex_app
# COPY --from=build /app/.env /app/.env

WORKDIR /app

CMD ["knex_app"]
