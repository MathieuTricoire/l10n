use l10n::fluent_bundle::FluentArgs;
use l10n::message;
use l10n::unic_langid::langid;

l10n::init!();

fn main() {
    let busy = message!(
        "home",
        "state.busy",
        "reason" = "Working",
        ...
    );

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
