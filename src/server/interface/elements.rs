use std::{marker::PhantomData, time::Duration};

use serde::Serialize;
use strum::{EnumString, VariantNames};

use crate::server::interface::{HasSummary, NoSummary, OodAction, responses::OodOptional};

pub struct OodMemWrite; // get unique device id (persistent)
impl OodAction for OodMemWrite {
    const NAME: &'static str = "mem_write";
    type Item = str; // file name
    type Reply = ();
    type ActionType = HasSummary<str>; // file data
}

pub struct OodMemRead; // get unique device id (persistent)
impl OodAction for OodMemRead {
    const NAME: &'static str = "mem_read";
    type Item = str; // file name
    type Reply = OodOptional<str>;
    type ActionType = NoSummary;
}

pub struct OodMemDelete; // get unique device id (persistent)
impl OodAction for OodMemDelete {
    const NAME: &'static str = "mem_delete";
    type Item = str; // file name
    type Reply = ();
    type ActionType = NoSummary;
}

// this is **NOT** the same as b.redirect() -> that is an *internal* redirect, this action instructs the device to open this URI in whatever external application
pub struct OodOpenUri;
impl OodAction for OodOpenUri {
    const NAME: &'static str = "uri";
    type Item = str;
    type Reply = (); // iOS shortcuts won't forget this, but don't leave me hanging on other things!!
    type ActionType = NoSummary;
}

pub struct OodInfo;
impl OodAction for OodInfo {
    const NAME: &'static str = "info";
    type Item = str; // interesting! we do this here, because we always use &Item (with &str it would become &&str)
    type Reply = ();
    type ActionType = HasSummary<str>;
}

pub struct OodButtonList<T>(PhantomData<T>);

impl<T> OodAction for OodButtonList<T>
where
    T: Serialize + AsRef<str>,
{
    const NAME: &'static str = "button";
    type Item = [T]; // (name, return value)
    type Reply = str; // shortcut limitation/simplification - no text back is an error (i.e., not optional)
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
    type Reply = ();
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
    type Reply = str;
    type ActionType = NoSummary;
}

pub struct OodTextInput;

impl OodAction for OodTextInput {
    const NAME: &'static str = "text_input";
    type Item = str; // default value (if editing)
    type Reply = OodOptional<str>; // shortcut limitation/simplification
    type ActionType = HasSummary<str>;
}
