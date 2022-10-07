use l10n::fluent_bundle::FluentArgs;
use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let busy = State::Busy("Working".to_string());

    assert_eq!(
        busy.translate(&langid!("en")),
        "Busy (\u{2068}Working\u{2069})"
    );

    let mut args = FluentArgs::new();
    args.set("gender", "male");
    assert_eq!(
        busy.translate_with_args(&langid!("fr"), Some(&args)),
        "\u{2068}Occupé\u{2069} (\u{2068}Working\u{2069})"
    );
}

#[derive(L10nMessage)]
#[l10n_message("home", "state")]
enum State {
    #[l10n_message(".busy", "reason" = .0, ...)]
    Busy(String),
}
