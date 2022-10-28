use l10n::L10nMessage;

l10n::init!();

fn main() {
    let _ = l10n::message!("home", "greeting");
}

#[derive(L10nMessage)]
#[l10n_message("home", "greeting")]
struct Welcome {}

#[derive(L10nMessage)]
#[l10n_message("home")]
enum State {
    #[l10n_message("online")]
    Online,
}

#[derive(L10nMessage)]
#[l10n_message("home", "status")]
enum Status {
    #[l10n_message(".online")]
    Online,
}
