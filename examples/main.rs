use l10n::fluent_bundle::{FluentArgs, FluentValue};
use l10n::unic_langid::langid;
use l10n::L10nMessage;
use std::borrow::Cow;

fn l10n_transform(s: &str) -> Cow<str> {
    Cow::from(s.replace("Occupée", "OcCuPéE🚫"))
}

fn time<'a>(positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
    match positional.get(0) {
        Some(FluentValue::String(s)) => FluentValue::String(Cow::from(format!("{}🕒", s))),
        Some(v) => v.to_owned(),
        _ => FluentValue::Error,
    }
}

l10n::init!({
    use_isolating: false, // Not recommended
    transform: Some(l10n_transform),
    functions: {
        "TIME": time
    }
});

fn main() {
    let lang = langid!("fr");
    let status = Status::BusyFor {
        reason: "Meeting",
        gender: Gender::Female,
        time: Time::minutes(30),
    };
    assert_eq!(status.translate(&lang), "OcCuPéE🚫 (Meeting) [30m🕒]");
}

#[derive(L10nMessage)]
#[l10n_message('a, "settings", "status")]
pub enum Status<'a, T>
where
    &'a T: 'a + Into<FluentValue<'a>>,
{
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy", "reason" = *.0, "gender" = .1)]
    Busy(&'a str, Gender),
    #[l10n_message(".busy-for", *reason, gender, time)]
    BusyFor {
        reason: &'a str,
        gender: Gender,
        time: T,
    },
    #[l10n_message(transparent)]
    Another(#[l10n_from] Other),
}

#[derive(L10nMessage)]
#[l10n_message("settings", "status.online")]
pub struct Other;

pub enum Gender {
    Female,
    Male,
    Other,
}

impl<'a> From<&'a Gender> for FluentValue<'a> {
    fn from(val: &'a Gender) -> Self {
        FluentValue::String(Cow::from(match val {
            Gender::Female => "female",
            Gender::Male => "male",
            Gender::Other => "other",
        }))
    }
}

pub struct Time(usize);

impl Time {
    pub fn minutes(minutes: usize) -> Time {
        Time(minutes)
    }
}

impl<'a> From<&'a Time> for FluentValue<'a> {
    fn from(val: &'a Time) -> Self {
        FluentValue::String(Cow::from(format!("{}m", val.0)))
    }
}
