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
