CREATE VIEW despesa_com_deputado AS
    SELECT expenses.id AS expense_id,
           expenses.data_emissao,
           expenses.fornecedor,
           expenses.valor_liquido,
           expenses.url_documento,
           deputados.nome,
           deputados.cpf,
           deputados.uf
    FROM expenses INNER JOIN deputados
    ON expenses.deputado_id = deputados.id;