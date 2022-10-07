use l10n::message;
use l10n::unic_langid::langid;

l10n::init!();

fn main() {
    let welcome = message!(
        "home",
        "welcome",
        "first-name" = "Alan",
        "last-name" = "Turing"
    );

    assert_eq!(
        welcome.translate(&langid!("en")),
        "Welcome \u{2068}Alan\u{2069} on Chat App!"
    );
    assert_eq!(
        welcome.translate(&langid!("fr")),
        "Bienvenue \u{2068}Alan\u{2069} \u{2068}Turing\u{2069} sur chat app."
    );
}
