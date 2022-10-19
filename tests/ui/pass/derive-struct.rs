use l10n::unic_langid::langid;
use l10n::L10nMessage;

l10n::init!();

fn main() {
    let welcome = Welcome {
        first_name: "Alan".to_string(),
        last_name: "Turing".to_string(),
    };

    assert_eq!(
        welcome.translate(&langid!("en")),
        "Welcome \u{2068}Alan\u{2069} on Chat App!"
    );
    assert_eq!(
        welcome.translate(&langid!("fr")),
        "Bienvenue \u{2068}Alan\u{2069} \u{2068}Turing\u{2069} sur chat app."
    );
}

#[derive(L10nMessage)]
#[l10n_message("home", "welcome", "first-name" = first_name.as_str(), "last-name" = last_name.as_str())]
struct Welcome {
    first_name: String,
    last_name: String,
}
