use l10n::fluent_bundle::{memoizer::MemoizerKind, FluentArgs, FluentValue};
use l10n::unic_langid::langid;
use l10n::L10nMessage;
use std::borrow::Cow;

fn transform(s: &str) -> Cow<str> {
    Cow::from(s.replace("l", "(ŀ)"))
}

fn formatter<M: MemoizerKind>(num: &FluentValue, _intls: &M) -> Option<String> {
    match num {
        FluentValue::Number(n) => Some(format!("{}_f64", n.value)),
        _ => None,
    }
}

fn lowercase<'a>(positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
    match positional.get(0) {
        Some(FluentValue::String(n)) => FluentValue::String(Cow::from(n.to_lowercase())),
        Some(v) => v.to_owned(),
        _ => FluentValue::Error,
    }
}

l10n::init!({
    transform: Some(transform),
    formatter: Some(formatter),
    use_isolating: false,
    functions: {
        "LOWERCASE": lowercase,
        "UPPERCASE": |positional, _named| -> FluentValue<'_> {
            match positional.get(0) {
                Some(FluentValue::String(n)) => FluentValue::String(Cow::from(n.to_uppercase())),
                Some(v) => v.to_owned(),
                _ => FluentValue::Error,
            }
        }

    }
});

fn main() {
    let first_name = "Alan";
    let last_name = "Turing";
    let points = 1000;

    let welcome = l10n::message!("home", "welcome", first_name, last_name, points);
    assert_eq!(
        welcome.translate(&langid!("en")),
        "We(ŀ)come alan TURING on Chat App, you have un(ŀ)ocked 1000_f64 points!"
    );
    assert_eq!(
        welcome.translate(&langid!("fr")),
        "Bienvenue alan TURING sur chat app, vous avez déb(ŀ)oqué 1000_f64 points !"
    );
}
