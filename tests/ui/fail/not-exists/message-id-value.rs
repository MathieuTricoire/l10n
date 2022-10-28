use l10n::L10nMessage;

l10n::init!();

fn main() {
    let _ = l10n::message!("home", "state");
}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
struct Busy {}

#[derive(L10nMessage)]
#[l10n_message("home")]
enum State {
    #[l10n_message("state")]
    Online,
    #[l10n_message("state.offline")]
    Offline,
}
