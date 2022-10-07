use l10n::message;

l10n::init!();

fn main() {
    let _ = message!("resource", "baz");
}
