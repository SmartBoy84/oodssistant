// to enforce guarantees about communication (i.e., wait on in and always give an out)

use std::{borrow::Cow, marker::PhantomData};

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

impl OodBridge {
    pub fn new(out_tx: mpsc::Sender<OodReplyType>, in_rx: mpsc::Receiver<OodPayload>) -> Self {
        Self { out_tx, in_rx }
    }

    pub async fn err_wrapper<T, A: OodAction>(
        &mut self,
        r: Result<T, IntOodAppErr<A>>,
    ) -> Result<T, IntOodAppErr<A>> {
        if let Err(ref e) = r {
            self.tx(OodReplyType::Err(e.into())).await;
        }
        r
    }
    async fn tx(&mut self, payload: OodReplyType) {
        self.out_tx.send(payload).await.expect("channel closed");
    }
    pub async fn rx(&mut self) -> OodPayload {
        self.in_rx.recv().await.expect("channel closed")
    }

    // all subsequent comms are: out -> in
    pub async fn cf<'a, A: OodAction>(
        &mut self,
        payload: &OodReply<'a, A>,
    ) -> Result<OodPayloadParser<A>, IntOodAppErr<A>> {
        let o = serde_json::to_string(payload).map_err(IntOodAppErr::InternalParseErr);
        let o = self.err_wrapper(o).await?;
        self.tx(OodReplyType::Payload(o.into()));

        let i = self.rx().await;
        Ok(OodPayloadParser {
            bridge: self,
            inner: i,
            _target: PhantomData,
        })
    }

    pub async fn external_redirect(mut self, uri: Cow<'static, str>) -> OodFinished {
        self.tx(OodReplyType::ExternalRedirect(uri)).await;
        OodFinished::new()
    }

    pub async fn internal_redirect(mut self, s_id: &SessionId) -> OodFinished {
        /* this allows for a pretty cool application: you can have pages that are only accessible through another page (not an actual route) */

        // consume the bridge because this sessions is DONE DOUGH!
        self.tx(OodReplyType::InternalRedirect(s_id.clone())).await;

        OodFinished::new()
    }

    pub async fn finished(mut self) -> OodFinished {
        self.tx(OodReplyType::Finished).await;

        OodFinished::new() // private - can only construct here
    }
}

// ("page name", next_step)
