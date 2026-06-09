use std::{borrow::Cow, fmt::Display};

pub mod test;

const SHORTCUT_URI: &str = "shortcuts"; // shortcuts://
const SHORTCUT_ACTION: &str = "run-shortcut";
const SHORTCUT_TEXT_ACTION: &str = "text";
const SHORTCUT_TEXT_FIELD: &str = "text";

pub struct OodShortcut<'a> {
    name: &'static str,
    input: Cow<'a, str>, // other types supported, but I only need text
}

impl<'a> Display for OodShortcut<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{SHORTCUT_URI}://{SHORTCUT_ACTION}?name={}&input={SHORTCUT_TEXT_ACTION}&{SHORTCUT_TEXT_FIELD}={}",
            self.name, self.input
        )
    }
}

impl<'a> OodShortcut<'a> {
    fn new(name: &'static str, input: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name,
            input: input.into(),
        }
    }
}
