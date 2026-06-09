use std::fmt::Display;

pub mod test;

const SHORTCUT_URI: &str = "shortcuts"; // shortcuts://
const SHORTCUT_ACTION: &str = "run-shortcut";
const SHORTCUT_TEXT_ACTION: &str = "text";
const SHORTCUT_TEXT_FIELD: &str = "text";

pub struct OodShortcut {
    name: &'static str,
    input: String, // other types supported, but I only need text
}

impl Display for OodShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{SHORTCUT_URI}://{SHORTCUT_ACTION}?name={}&input={SHORTCUT_TEXT_ACTION}&{SHORTCUT_TEXT_FIELD}={}",
            self.name, self.input
        )
    }
}

impl OodShortcut {
    fn new(name: &'static str, input: String) -> Self {
        Self { name, input }
    }
}
