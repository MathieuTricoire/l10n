use l10n::{message, L10nMessage};

l10n::init!();

fn main() {
    let _ = message!("home", "welcome.attribute");
}

#[derive(L10nMessage)]
#[l10n_message("home", "welcome.attribute")]
struct Welcome {}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum State {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".other")]
    Other,
}
