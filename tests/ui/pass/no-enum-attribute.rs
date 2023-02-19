use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let state = State::Offline;
    assert_eq!(state.translate(&langid!("en")), "Offline");
    assert_eq!(state.translate(&langid!("fr")), "Hors ligne");
}

#[derive(L10nMessage)]
enum State {
    #[l10n_message("home", "state.online")]
    Online,
    #[l10n_message("home", "state.offline")]
    Offline,
    #[l10n_message("home", "state.busy", "reason" = reason.as_str(), "gender" = "other")]
    Busy { reason: String },
}
