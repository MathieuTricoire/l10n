use unic_langid::LanguageIdentifier;

pub fn locales_to_string(locales: &[LanguageIdentifier], separator: &str) -> String {
    locales
        .iter()
        .map(|locale| locale.to_string())
        .collect::<Vec<_>>()
        .join(separator)
}

pub fn values_to_string<T: ToString>(values: &[T], separator: &str) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(separator)
}

pub fn for_locales(values: &[LanguageIdentifier]) -> String {
    format!(
        "for {}: {}",
        grammar_number(values, "locale", "locales"),
        locales_to_string(values, ", "),
    )
}

pub fn grammar_number<T, S: ToString>(values: &[T], singular: S, plural: S) -> S {
    if values.len() == 1 {
        singular
    } else {
        plural
    }
}
