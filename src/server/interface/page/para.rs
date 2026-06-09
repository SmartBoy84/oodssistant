use std::str::FromStr;

use warp::Filter;

use crate::server::interface::page::{
    OodPage, OodPageHandler, OodPagePara, OodPageSession, basic::OodStaticPageHandler,
};

pub struct OodPara<T: OodParaPage>(pub T);
impl OodPagePara for String {} // this for sure

pub trait OodParaPage: OodPageSession<Self::Para> {
    const URI: &'static str; // icb implementing dynamic url, will do so when/if I need it
    type Para: OodPagePara + FromStr;

    // mostly for external teating
    fn create_uri(para: &str) -> String {
        format!("{}/{para}", Self::URI)
    }
}

impl<P> OodPage for OodPara<P>
where
    P: OodParaPage + Send + Sync + 'static,
{
    type ParaSettings = &'static str; // i.e., the URI
    type Para = P::Para;
    type PageSession = P;
    type ParaHandler = OodParaPageHandler;

    fn split(self) -> (Self::PageSession, Self::ParaSettings) {
        (self.0, P::URI)
    }
}

pub struct OodParaPageHandler;
impl<P> OodPageHandler<&'static str, P> for OodParaPageHandler
where
    P: OodPagePara + FromStr + 'static,
{
    fn para_extractor(
        settings: &'static str,
    ) -> impl warp::Filter<Extract = (P,), Error = warp::Rejection> + Clone + Send {
        // hella simple - we can re-use the first bit
        OodStaticPageHandler::para_extractor(settings)
            .untuple_one() // ignore the () return from the other path
            .and(warp::path::param::<P>())
    }
}
