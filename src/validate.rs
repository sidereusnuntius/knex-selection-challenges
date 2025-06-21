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