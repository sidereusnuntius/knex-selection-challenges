CREATE TABLE expenses (
    id SERIAL PRIMARY KEY,
    data_emissao DATE,
    fornecedor VARCHAR NOT NULL,
    valor_liquido REAL NOT NULL,
    url_documento VARCHAR,
    deputado_id INTEGER NOT NULL REFERENCES deputados(id)
)