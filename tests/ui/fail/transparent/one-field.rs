use l10n::L10nMessage;

l10n::init!();

fn main() {}

#[derive(L10nMessage)]
#[l10n_message(transparent)]
struct StructMissingField;

#[derive(L10nMessage)]
#[l10n_message(transparent)]
struct StructTooManyUnnamedFields(Welcome, String);

#[derive(L10nMessage)]
#[l10n_message(transparent)]
struct StructTooManyNamedFields {
    field_1: Welcome,
    field_2: String,
}

#[derive(L10nMessage)]
enum EnumMissingField {
    #[l10n_message(transparent)]
    MissingField,
}

#[derive(L10nMessage)]
enum EnumTooManyUnnamedFields {
    #[l10n_message(transparent)]
    TooManyUnnamedFields(Welcome, String),
}

#[derive(L10nMessage)]
enum EnumTooManyNamedFields {
    #[l10n_message(transparent)]
    TooManyNamedFields { field_1: Welcome, field_2: String },
}

#[derive(L10nMessage)]
#[l10n_message("home", "welcome", "first-name" = first_name.as_str(), "last-name" = last_name.as_str())]
struct Welcome {
    first_name: String,
    last_name: String,
}
