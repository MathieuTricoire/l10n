use l10n::fluent_bundle::FluentValue;
use l10n::unic_langid::langid;
use l10n::{message_args, L10nMessage};

l10n::init!();

fn main() {
    let online: State<'_, usize, usize> = State::Online;
    assert_eq!(online.translate(&langid!("en")), "Online");
    assert_eq!(online.translate(&langid!("fr")), "En ligne");

    let busy_for = State::BusyFor {
        reason: "Working",
        hours: 2,
        gender: "female",
    };
    assert_eq!(
        busy_for.translate(&langid!("en")),
        "Busy for \u{2068}\u{2068}2\u{2069} hours\u{2069} (\u{2068}Working\u{2069})"
    );
    assert_eq!(
        busy_for.translate(&langid!("fr")),
        "\u{2068}Occupée\u{2069} pour \u{2068}\u{2068}2\u{2069} heures\u{2069} (\u{2068}Working\u{2069})"
    );

    // Change some arguments
    assert_eq!(
        busy_for.translate_with_args(&langid!("en"), Some(&message_args!("hours" => 3))),
        "Busy for \u{2068}\u{2068}3\u{2069} hours\u{2069} (\u{2068}Working\u{2069})"
    );
    assert_eq!(
        busy_for.translate_with_args(
            &langid!("fr"),
            Some(&message_args!("hours" => 3, "gender" => "male"))
        ),
        "\u{2068}Occupé\u{2069} pour \u{2068}\u{2068}3\u{2069} heures\u{2069} (\u{2068}Working\u{2069})"
    );
}

#[derive(L10nMessage)]
#[l10n_message('a, "home", "state")]
enum State<'a, T, U>
where
    &'a T: 'a + Into<FluentValue<'a>>,
    U: 'a + Into<FluentValue<'a>> + Clone,
{
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".busy-for", "reason" = *reason, hours, "gender" = gender.clone())]
    BusyFor {
        reason: &'a str,
        hours: T,
        gender: U,
    },
}
