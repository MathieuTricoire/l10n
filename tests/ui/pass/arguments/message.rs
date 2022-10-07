use l10n::unic_langid::langid;
use l10n::{message, message_args};

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

    // Overriding
    assert_eq!(
        welcome.translate_with_args(&langid!("en"), Some(&message_args!("first-name" => "John"))),
        "Welcome \u{2068}John\u{2069} on Chat App!"
    );
    assert_eq!(
        welcome.translate_with_args(&langid!("fr"), Some(&message_args!("first-name" => "John"))),
        "Bienvenue \u{2068}John\u{2069} \u{2068}Turing\u{2069} sur chat app."
    );
}
