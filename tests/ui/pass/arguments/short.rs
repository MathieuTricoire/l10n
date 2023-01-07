use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let reason = "Working".to_string();
    let gender = "male";
    let msg = l10n::message!("home", "state.busy", reason, gender);
    assert_eq!(
        msg.translate(&langid!("en")),
        "Busy (\u{2068}Working\u{2069})"
    );
    assert_eq!(
        msg.translate(&langid!("fr")),
        "\u{2068}Occupé\u{2069} (\u{2068}Working\u{2069})"
    );
}
