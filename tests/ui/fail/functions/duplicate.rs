use l10n::fluent_bundle::{FluentArgs, FluentValue};

fn fake_function<'a>(_positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
    FluentValue::None
}

l10n::init!({
    functions: {
        "NUMBER": fake_function,
        "DAY": fake_function,
        "TIME": fake_function,
        "DAY": fake_function,
    }
});

fn main() {}
