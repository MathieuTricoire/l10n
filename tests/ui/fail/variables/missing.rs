use l10n::{message, L10nMessage};

l10n::init!();

fn main() {
    let _ = message!("home", "welcome");
    let _ = message!("home", "welcome", "extra" = "value");
}

#[derive(L10nMessage)]
#[l10n_message("home", "welcome")]
struct WelcomeNoArgs {}

#[derive(L10nMessage)]
#[l10n_message("home", "welcome", "first-name" = "Alan", "extra" = "value")]
struct WelcomeMissingArgs {}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum StateNoArgs {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy")]
    Busy,
    #[l10n_message("state.busy")]
    BusyTodo,
}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum StateMissingArgs {
    #[l10n_message(".online", "extra" = "value")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy", "reason" = "Working", "extra" = "value")]
    Busy,
    #[l10n_message("state.busy", "reason" = "Working", "extra" = "value")]
    BusyTodo,
}
