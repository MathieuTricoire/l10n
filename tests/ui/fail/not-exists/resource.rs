use l10n::{message, L10nMessage};

l10n::init!();

fn main() {
    let _ = message!("unknown", "welcome");
}

#[derive(L10nMessage)]
#[l10n_message("unknown", "welcome")]
struct Welcome {}

#[derive(L10nMessage)]
#[l10n_message("unknown", "state")]
enum State {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message("another-unknown", "other")]
    Other,
}
