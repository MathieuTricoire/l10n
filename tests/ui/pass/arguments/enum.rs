use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let state = State::Busy {
        reason: "Working".to_string(),
    };
    assert_eq!(
        state.translate(&langid!("en")),
        "Busy (\u{2068}Working\u{2069})"
    );
    assert_eq!(
        state.translate(&langid!("fr")),
        "\u{2068}Non disponible\u{2069} (\u{2068}Working\u{2069})"
    );

    let state = StateAlternative::Busy {
        reason: "Working".to_string(),
        gender: "female".to_string(),
    };
    assert_eq!(
        state.translate(&langid!("en")),
        "Busy (\u{2068}Working\u{2069})"
    );
    assert_eq!(
        state.translate(&langid!("fr")),
        "\u{2068}Occupée\u{2069} (\u{2068}Working\u{2069})"
    );
}

#[derive(L10nMessage)]
#[l10n_message("home", "state", "gender" = "other")]
enum State {
    #[l10n_message(".busy", "reason" = reason.as_str())]
    Busy { reason: String },
}

#[derive(L10nMessage)]
#[l10n_message("home", "gender" = "other")]
enum StateAlternative {
    #[l10n_message("state.busy", "reason" = reason.as_str(), "gender" = gender.as_str())]
    Busy { reason: String, gender: String },
}
