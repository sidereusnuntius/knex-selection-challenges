// @generated automatically by Diesel CLI.

diesel::table! {
    deputados (id) {
        id -> Int4,
        nome -> Varchar,
        #[max_length = 2]
        uf -> Bpchar,
        cpf -> Varchar,
        partido -> Nullable<Varchar>,
    }
}

diesel::table! {
    expenses (id) {
        id -> Int4,
        data_despesa -> Date,
        fornecedor -> Varchar,
        valor_liquido -> Float4,
        url_documento -> Nullable<Varchar>,
        deputado_id -> Int4,
    }
}

diesel::joinable!(expenses -> deputados (deputado_id));

diesel::allow_tables_to_appear_in_same_query!(
    deputados,
    expenses,
);
