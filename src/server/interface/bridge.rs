// to enforce guarantees about communication (i.e., wait on in and always give an out)

use std::{borrow::Cow, marker::PhantomData};

use tokio::sync::mpsc;

use crate::server::{
    SessionId,
    interface::{
        IntOodAppErr, IntOodParseErr, OodAction, OodActionType, OodReplyType,
        external::{OodPayloadParser, OodResponse},
        internal::LinkedOodReply,
    },
    request::{GenericResult, OodPayloadStreamer},
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
    in_rx: mpsc::Receiver<Result<OodResponse, Box<dyn std::error::Error + Sync + Send>>>,
}

pub struct OodReq<'a, A: OodAction> {
    b: &'a mut OodBridge,
    raw: LinkedOodReply<A>, // immutable, ref-counted buffer
}

/*
Motivation:
1. Bridge::n() -> returns OodReq => can save this somewhere to avoid repeated serialisation
2. However separating into OodReq allows also for my_bridge.n(...).cf(...).p(...) to be run! So you go from OodBridge -> OodReq (parse request) => OodPayloadParser (get reply) -> T (parse reply)
*/
impl<'a, A: OodAction> OodReq<'a, A> {
    // all subsequent comms are: out -> in
    pub async fn c(&'a mut self) -> Result<OodPayloadParser<A>, IntOodAppErr<A>> {
        self.b.comm(&self.raw).await
    }
}

impl OodBridge {
    pub fn new(
        out_tx: mpsc::Sender<OodReplyType>,
        in_rx: mpsc::Receiver<GenericResult<OodResponse>>,
    ) -> Self {
        Self { out_tx, in_rx }
    }

    async fn tx(&self, payload: OodReplyType) {
        self.out_tx.send(payload).await.expect("channel closed"); // channel closure is a BUG so treat it as such
    }
    pub async fn rx(&mut self) -> GenericResult<OodResponse> {
        self.in_rx.recv().await.expect("channel closed")
    }
    pub async fn comm<A: OodAction>(
        &mut self,
        payload: &LinkedOodReply<A>,
    ) -> Result<OodPayloadParser<A>, IntOodAppErr<A>> {
        // foundational communication method - in -> out -> ...
        self.tx(OodReplyType::Payload(payload.inner())).await; // cloning bytes is cheap - increment ref count
        let inner = self.rx().await.map_err(|e| e.downcast::<<<A::ActionType as OodActionType>::Data as OodPayloadStreamer>::StreamErr>().expect("failed downcasting")).map_err(IntOodParseErr::PayloadErr).map_err(IntOodAppErr::InternalParseErr)?;
        Ok(OodPayloadParser::new(inner))
    }

    pub async fn cf<A>(
        &mut self,
        payload: &LinkedOodReply<A>,
    ) -> Result<OodPayloadParser<A>, IntOodAppErr<A>>
    where
        A: OodAction,
    {
        self.comm(payload).await
    }

    pub async fn n<A: OodAction>(&mut self, raw: &LinkedOodReply<A>) -> OodReq<'_, A> {
        OodReq {
            b: self,
            raw: raw.clone(),
        }
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
