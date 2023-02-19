use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let message = l10n::message!("app", "profile");
    let profile = message.translate(&langid!("en"));
    assert_eq!(profile, "test");
}
