use l10n::message;

l10n::init!();

fn main() {
    let _ = message!(
        "home",
        "state.busy",
        "reason" = "Working",
        ...
    );
}
