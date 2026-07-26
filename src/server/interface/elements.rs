use std::{marker::PhantomData, time::Duration};

use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

use crate::server::interface::{HasSummary, NoSummary, OodAction};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OptionalResponse<T> {
    Empty(EmptyResponse), // AYYYY, put this *first* to have "" -> EmptyString
    Res(T),
}

#[derive(Debug)]
pub enum EmptyResponse {
    Null,
    None,
    EmptyString,
}

#[derive(Debug)]
#[repr(transparent)] // thus, dereferencing to &str is SAFE
pub struct OodFilePath(pub str);
/*
iOS Shortcuts is so annoying - when writing it automatically adds the .txt extension but when reading it doesn't
this is to avoid errors by statically enforcing the addition of a `.txt` on all paths
*/

pub struct OodWrite; // get unique device id (persistent)
impl OodAction for OodWrite {
    const NAME: &'static str = "write";
    type Item = str; // file name
    type Reply = EmptyResponse;
    type ActionType = HasSummary<OodFilePath>; // file data
}

pub struct OodRead; // get unique device id (persistent)
impl OodAction for OodRead {
    const NAME: &'static str = "read";
    type Item = OodFilePath; // file name
    type Reply = OptionalResponse<String>;
    type ActionType = NoSummary;
}

// this is **NOT** the same as b.redirect() -> that is an *internal* redirect, this action instructs the device to open this URI in whatever external application
pub struct OodOpenUri;
impl OodAction for OodOpenUri {
    const NAME: &'static str = "uri";
    type Item = str;
    type Reply = EmptyResponse; // iOS shortcuts won't forget this, but don't leave me hanging on other things!!
    type ActionType = NoSummary;
}

pub struct OodInfo;
impl OodAction for OodInfo {
    const NAME: &'static str = "info";
    type Item = str; // interesting! we do this here, because we always use &Item (with &str it would become &&str)
    type Reply = EmptyResponse;
    type ActionType = HasSummary<str>;
}

pub struct OodButtonList<T>(PhantomData<T>);

impl<T> OodAction for OodButtonList<T>
where
    T: Serialize + AsRef<str>,
{
    const NAME: &'static str = "button";
    type Item = [T]; // (name, return value)
    type Reply = String; // shortcut limitation/simplification
    type ActionType = HasSummary<str>;
}
pub struct OodTimer; // start a timer on the device
#[derive(Serialize)]
pub struct Seconds(u64);
impl From<Duration> for Seconds {
    fn from(value: Duration) -> Self {
        Self(value.as_secs())
    }
}

impl OodAction for OodTimer {
    const NAME: &'static str = "timer";
    type Item = Option<Seconds>; // None - deactivate timer
    type Reply = EmptyResponse;
    type ActionType = NoSummary;
}

pub struct OodStopwatch; // start/stop/reset stopwatch on device
#[derive(Debug, VariantNames, EnumString, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OodStopwatchAction {
    Start,
    Reset,
    Stop,
}
impl OodAction for OodStopwatch {
    const NAME: &'static str = "stopwatch";
    type Item = OodStopwatchAction;
    type Reply = EmptyResponse;
    type ActionType = NoSummary;
}

pub struct OodTextInput;

impl OodAction for OodTextInput {
    const NAME: &'static str = "text_input";
    type Item = str; // default value (if editing)
    type Reply = String; // shortcut limitation/simplification
    type ActionType = HasSummary<str>;
}
