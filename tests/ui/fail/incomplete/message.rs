l10n::init!();

fn main() {
    let _ = l10n::message!(
        "home",
        "state.busy",
        "reason" = "Working",
        ...
    );
}
