use std::{borrow::Cow, marker::PhantomData, time::Duration};

use serde::Serialize;
use strum::IntoStaticStr;

use crate::server::interface::{
    external::responses::{ImageWrapper, OodOptional},
    internal::{
        HasData, NoData, OodAction,
        items::{JsonItemWrap, OodDisplayItem, OodEnumItem, OodOptionalItem},
        payloads::SharedBytes,
    },
};

#[derive(Debug, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum OodCameraSide {
    Front,
    Back,
}

pub struct OodTakeImage;
impl OodAction for OodTakeImage {
    const NAME: &'static str = "image";
    type Item = OodEnumItem<OodCameraSide>;
    type Reply = ImageWrapper;
    type ActionType = NoData;
}

pub struct OodMemWrite; // get unique device id (persistent)
impl OodAction for OodMemWrite {
    const NAME: &'static str = "mem_write";
    type Item = Cow<'static, str>; // file name
    type Reply = ();
    type ActionType = HasData<SharedBytes<str>>; // file data
}

pub struct OodMemRead; // get unique device id (persistent)
impl OodAction for OodMemRead {
    const NAME: &'static str = "mem_read";
    type Item = Cow<'static, str>; // file name
    type Reply = OodOptional<String>;
    type ActionType = NoData;
}

pub struct OodMemDelete; // get unique device id (persistent)
impl OodAction for OodMemDelete {
    const NAME: &'static str = "mem_delete";
    type Item = Cow<'static, str>; // file name
    type Reply = ();
    type ActionType = NoData;
}

// this is **NOT** the same as b.redirect() -> that is an *internal* redirect, this action instructs the device to open this URI in whatever external application
pub struct OodOpenUri;
impl OodAction for OodOpenUri {
    const NAME: &'static str = "uri";
    type Item = Cow<'static, str>;
    type Reply = (); // iOS shortcuts won't forget this, but don't leave me hanging on other things!!
    type ActionType = NoData;
}

pub struct OodInfo;
impl OodAction for OodInfo {
    const NAME: &'static str = "info";
    type Item = Cow<'static, str>; // interesting! we do this here, because we always use &Item<'a> (with &str it would become &&str)
    type Reply = ();
    type ActionType = HasData<SharedBytes<str>>;
}

pub struct OodButtonList<T>(PhantomData<fn(&T)>);

impl<T> OodAction for OodButtonList<T>
where
    T: Serialize,
{
    const NAME: &'static str = "button";
    type Item = JsonItemWrap<[T]>;
    type Reply = String; // shortcut limitation/simplification - no text back is an error (i.e., not optional)
    type ActionType = HasData<SharedBytes<str>>;
}

pub struct OodTimer; // start a timer on the device
#[derive(derive_more::Display)]
pub struct Seconds(u64);
impl From<Duration> for Seconds {
    fn from(value: Duration) -> Self {
        Self(value.as_secs())
    }
}

impl OodAction for OodTimer {
    const NAME: &'static str = "timer";
    type Item = OodOptionalItem<OodDisplayItem<Seconds>>; // None - deactivate timer
    type Reply = ();
    type ActionType = NoData;
}

pub struct OodStopwatch; // start/stop/reset stopwatch on device
#[derive(Debug, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum OodStopwatchAction {
    Start,
    Reset,
    Stop,
}
impl OodAction for OodStopwatch {
    const NAME: &'static str = "stopwatch";
    type Item = OodEnumItem<OodStopwatchAction>;
    type Reply = String;
    type ActionType = NoData;
}

pub struct OodTextInput;

impl OodAction for OodTextInput {
    const NAME: &'static str = "text_input";
    type Item = Cow<'static, str>; // default value (if editing)
    type Reply = OodOptional<String>; // shortcut limitation/simplification
    type ActionType = HasData<SharedBytes<str>>;
}
