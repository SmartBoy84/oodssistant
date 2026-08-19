// to enforce guarantees about communication (i.e., wait on in and always give an out)

use std::{borrow::{Borrow, Cow}, marker::PhantomData};

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
    pub async fn comm<'a, A: OodAction>(
        &mut self,
        payload: &LinkedOodReply<A>,
    ) -> Result<OodPayloadParser<A>, IntOodAppErr<A>>
    where
        A: 'a,
    {
        // foundational communication method - in -> out -> ...
        self.tx(OodReplyType::Payload(payload.inner())).await; // cloning bytes is cheap - increment ref count
        let inner = self.rx().await.map_err(|e| e.downcast::<<<A::ActionType as OodActionType>::Data as OodPayloadStreamer>::StreamErr>().expect("failed downcasting")).map_err(IntOodParseErr::PayloadErr).map_err(IntOodAppErr::InternalParseErr)?;
        Ok(OodPayloadParser::new(inner))
    }

    pub async fn cf<A, T>(
        &mut self,
        payload: T,
    ) -> Result<OodPayloadParser<A>, IntOodAppErr<A>>
    where
        A: OodAction,
        T: Borrow<LinkedOodReply<A>>
    {
        self.comm(payload.borrow()).await
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
