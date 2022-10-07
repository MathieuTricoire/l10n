use l10n::L10nMessage;

l10n::init!();

fn main() {}

#[derive(L10nMessage)]
#[l10n_message("home")]
enum State {
    #[l10n_message(".online")]
    Online,
}
