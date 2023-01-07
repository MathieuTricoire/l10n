use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let welcome = l10n::message!(
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

    // Overriding
    assert_eq!(
        welcome.translate_with_args(
            &langid!("en"),
            Some(&l10n::message_args!("first-name" => "John"))
        ),
        "Welcome \u{2068}John\u{2069} on Chat App!"
    );
    assert_eq!(
        welcome.translate_with_args(
            &langid!("fr"),
            Some(&l10n::message_args!("first-name" => "John"))
        ),
        "Bienvenue \u{2068}John\u{2069} \u{2068}Turing\u{2069} sur chat app."
    );
}
