use l10n::{message, L10nMessage};

l10n::init!();

fn main() {
    let _ = message!(
        "home",
        "welcome",
        "first-name" = "Alan",
        "first-name" = "Turing"
    );
}

#[derive(L10nMessage)]
#[l10n_message("home", "welcome", "first-name" = "Alan", "first-name" = "Turing")]
struct Welcome {}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum State {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy", "reason" = .0, "reason" = .1)]
    Busy(String, String),
}
