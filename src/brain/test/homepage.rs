use std::sync::Arc;

use tokio::sync::Mutex;

use crate::server::{
    handlers::SessionId,
    interface::{
        OodAction, OodAppErr,
        bridge::{OodBridge, OodFinished},
        elements::{OodButtonList, OodInfo, OodTextInput},
        page::{OodPageSession, OodSessionPara, basic::OodBasicPage},
    },
};