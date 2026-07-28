// to enforce guarantees about communication (i.e., wait on in and always give an out)

use std::{borrow::Cow, marker::PhantomData};

use serde_json::Value;
use tokio::sync::mpsc;

use crate::server::{
    SessionId,
    interface::{IntOodAppErr, OodAction, OodPayload, OodPayloadParser, OodReply, OodReplyType},
};

// to enforce that b.finished() is called
pub struct OodFinished {
    _priv: PhantomData<()>,
}
impl OodFinished {
    fn new() -> Self {
        Self { _priv: PhantomData }
    }
}

pub struct OodBridge {
    out_tx: mpsc::Sender<OodReplyType>,
    in_rx: mpsc::Receiver<OodPayload>,
}

pub struct OodReq<'a, A: OodAction> {
    b: &'a mut OodBridge,
    raw: Value, // immutable, ref-counted buffer
    src: PhantomData<A>,
}

/*
Motivation:
1. Bridge::n() -> returns OodReq => can save this somewhere to avoid repeated serialisation
2. However separating into OodReq allows also for my_bridge.n(...).cf(...).p(...) to be run! So you go from OodBridge -> OodReq (parse request) => OodPayloadParser (get reply) -> T (parse reply)
*/
impl<'a, A: OodAction> OodReq<'a, A> {
    // all subsequent comms are: out -> in
    pub async fn c(&'a mut self) -> Result<OodPayloadParser<A>, IntOodAppErr<A>> {
        let i = self.b.comm(self.raw.clone()).await; // cloning bytes is cheap - increment ref count
        Ok(OodPayloadParser {
            inner: i,
            _target: PhantomData,
        })
    }
}

impl OodBridge {
    pub fn new(out_tx: mpsc::Sender<OodReplyType>, in_rx: mpsc::Receiver<OodPayload>) -> Self {
        Self { out_tx, in_rx }
    }

    async fn tx(&self, payload: OodReplyType) {
        self.out_tx.send(payload).await.expect("channel closed"); // channel closure is a BUG so treat it as such
    }
    pub async fn rx(&mut self) -> OodPayload {
        self.in_rx.recv().await.expect("channel closed")
    }
    pub async fn comm(&mut self, raw_payload: Value) -> OodPayload {
        // foundational communication method - in -> out -> ...
        self.tx(OodReplyType::Payload(raw_payload)).await; // cloning bytes is cheap - increment ref count
        self.rx().await
    }
    pub fn parse_payload<A: OodAction>(
        &self,
        payload: &OodReply<'_, A>,
    ) -> Result<Value, IntOodAppErr<A>> {
        serde_json::to_value(payload).map_err(IntOodAppErr::InternalParseErr)
    }

    pub async fn cf<A>(
        &mut self,
        payload: &OodReply<'_, A>,
    ) -> Result<OodPayloadParser<A>, IntOodAppErr<A>>
    where
        A: OodAction,
    {
        let raw = self.parse_payload(payload)?;
        let inner = self.comm(raw).await;
        Ok(OodPayloadParser {
            inner,
            _target: PhantomData,
        })
    }

    pub async fn n<'a, 'p, A: OodAction>(
        &'a mut self,
        payload: &OodReply<'_, A>,
    ) -> Result<OodReq<'a, A>, IntOodAppErr<A>> {
        let raw = self.parse_payload(payload)?;
        Ok(OodReq {
            b: self,
            raw,
            src: PhantomData,
        })
    }

    pub async fn external_redirect(self, uri: Cow<'static, str>) -> OodFinished {
        self.tx(OodReplyType::ExternalRedirect(uri)).await;
        OodFinished::new()
    }

    pub async fn internal_redirect(self, s_id: &SessionId) -> OodFinished {
        /* this allows for a pretty cool application: you can have pages that are only accessible through another page (not an actual route) */

        // consume the bridge because this sessions is DONE DOUGH!
        self.tx(OodReplyType::InternalRedirect(s_id.clone())).await;

        OodFinished::new()
    }

    pub async fn finished(self) -> OodFinished {
        self.tx(OodReplyType::Finished).await;

        OodFinished::new() // private - can only construct here
    }
}

// ("page name", next_step)
