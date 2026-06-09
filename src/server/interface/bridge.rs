// to enforce guarantees about communication (i.e., wait on in and always give an out)

use std::marker::PhantomData;

use tokio::sync::mpsc;

use crate::server::{
    handlers::SessionId,
    interface::{
        OodAction, OodAppErr, OodReply, OodReplyType, OodRes,
        page::{OodPagePara, OodPageSession},
        redirect::IntoOodInternalPayload,
    },
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
    in_rx: mpsc::Receiver<serde_json::Value>,
}

impl OodBridge {
    pub fn new(
        out_tx: mpsc::Sender<OodReplyType>,
        in_rx: mpsc::Receiver<serde_json::Value>,
    ) -> Self {
        Self { out_tx, in_rx }
    }

    // all subsequent comms are: out -> in
    async fn communicate<'a, A: OodAction>(
        &mut self,
        payload: &OodReply<'a, A>,
    ) -> Result<A::Reply, OodAppErr> {
        self.out_tx
            .send(OodReplyType::Payload(
                serde_json::to_string(&payload)
                    .map_err(|e| OodAppErr::InternalParseError(e))?
                    .into(),
            ))
            .await
            .expect("channel closed"); // in this land of sessions - panic is fine!

        serde_json::from_value::<OodRes<A>>(
            self.in_rx
                .recv()
                .await
                .inspect(|e| println!("{e:?}"))
                .expect("channel closed"),
        )
        .map_err(|e| OodAppErr::ExternalParseError(e))
        .map(|OodRes { res, .. }| res)
    }

    pub async fn cf<'a, A: OodAction>(
        &mut self,
        payload: &OodReply<'a, A>,
    ) -> Result<A::Reply, OodAppErr> {
        let r = self.communicate::<A>(payload).await;
        if let Err(ref e) = r {
            self.out_tx
                .send(OodReplyType::Error(e.to_string()))
                .await
                .expect("channel closed");
        }
        r
    }

    pub async fn external_redirect(self, s_id: SessionId) -> OodFinished {
        self.out_tx
            .send(OodReplyType::ExternalRedirect(s_id))
            .await
            .expect("channel closed");
        OodFinished::new()
    }

    pub async fn internal_redirect<P: OodPagePara, S: OodPageSession<P>>(
        self,
        s: S,
        p: P,
    ) -> OodFinished
    where
        S: 'static,
        P: 'static,
    {
        /* this allows for a pretty cool application: you can have pages that are only accessible through another page (not an actual route) */

        // consume the bridge because this sessions is DONE DOUGH!
        self.out_tx
            .send(OodReplyType::InternalRedirect(Box::new(
                s.into_internal_payload(p),
            )))
            .await
            .expect("channel closed");

        OodFinished::new()
    }

    pub async fn finished(self) -> OodFinished {
        self.out_tx
            .send(OodReplyType::Finished)
            .await
            .expect("channel closed");

        OodFinished::new() // private - can only construct here
    }
}

// ("page name", next_step)
