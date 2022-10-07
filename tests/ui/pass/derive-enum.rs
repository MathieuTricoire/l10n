use l10n::unic_langid::langid;
use l10n::{message_args, L10nMessage};

l10n::init!();

fn main() {
    let en = &langid!("en");

    let online = State::Online;
    assert_eq!(online.translate(en), "Online");

    let offline = State::Offline;
    assert_eq!(offline.translate(en), "Offline");

    let busy: State = Busy::NotTimed("Working".to_string(), "female".to_string()).into();
    assert_eq!(busy.translate(en), "Busy (\u{2068}Working\u{2069})");

    let busy_for: State = Busy::Timed(BusyFor {
        reason: "Working".to_string(),
        hours: 2,
        gender: "other".to_string(),
    })
    .into();
    assert_eq!(
        busy_for.translate(en),
        "Busy for \u{2068}\u{2068}2\u{2069} hours\u{2069} (\u{2068}Working\u{2069})"
    );

    let fr = &langid!("fr");

    let online = State::Online;
    assert_eq!(online.translate(fr), "En ligne");

    let offline = State::Offline;
    assert_eq!(offline.translate(fr), "Hors ligne");

    let busy: State = Busy::NotTimed("Travail".to_string(), "female".to_string()).into();
    assert_eq!(
        busy.translate(fr),
        "\u{2068}Occupée\u{2069} (\u{2068}Travail\u{2069})"
    );

    let busy_for: State = Busy::Timed(BusyFor {
        reason: "Travail".to_string(),
        hours: 2,
        gender: "other".to_string(),
    })
    .into();
    assert_eq!(
        busy_for.translate(fr),
        "\u{2068}Non disponible\u{2069} pour \u{2068}\u{2068}2\u{2069} heures\u{2069} (\u{2068}Travail\u{2069})"
    );

    let args = message_args!("gender" => "female", "hours" => 1);
    assert_eq!(
        busy_for.translate_with_args(fr, Some(&args)),
        "\u{2068}Occupée\u{2069} pour \u{2068}1 heure\u{2069} (\u{2068}Travail\u{2069})"
    );
}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum State {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(transparent)]
    Busy(#[l10n_from] Busy),
}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum Busy {
    #[l10n_message(".busy", "reason" = .0, "gender" = .1)]
    NotTimed(String, String),
    #[l10n_message(transparent)]
    Timed(#[l10n_from] BusyFor),
}

#[derive(L10nMessage)]
#[l10n_message("home", "state.busy-for", reason, hours, gender)]
struct BusyFor {
    reason: String,
    hours: usize,
    gender: String,
}
