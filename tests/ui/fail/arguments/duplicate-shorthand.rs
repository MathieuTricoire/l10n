use l10n::{message, L10nMessage};

l10n::init!();

fn main() {
    let reason = "reason".to_string();
    let _ = message!("home", "state.busy", reason, *reason,);
}

#[derive(L10nMessage)]
#[l10n_message("home", "state.busy", *reason, reason)]
struct Busy {
    reason: String,
}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum State {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy", reason, *reason)]
    Busy { reason: String },
}
