use l10n::L10nMessage;

l10n::init!();

fn main() {}

#[derive(L10nMessage)]
enum State {
    #[l10n_message("state.online")]
    Online,
    #[l10n_message("state.offline")]
    Offline,
}
