use warp::Filter;

use crate::server::interface::page::{OodPage, OodPageHandler, OodPageSession};

// now we define various "default handlers" that the page can inherit to avoid having to implement OodPageHandler manually
pub trait OodStaticPage: OodPageSession<()> {
    // this is for pages that have static URLs -> no input parameters from the path
    fn url(&self) -> &'static str; // but url can be derived from the page itself
}

pub trait OodBasicPage: OodPageSession<()> {
    // *most* basic page type - no parameters + URL is fixed
    const URI: &str;
}

impl<P: OodBasicPage> OodStaticPage for P {
    fn url(&self) -> &'static str {
        Self::URI
    }
}

impl<P> OodPage for P
where
    P: OodStaticPage + OodPageSession<()> + Send + Sync + 'static,
{
    // here's the trick to simplify it for the end user - Self is the page session!
    type ParaSettings = &'static str;
    type Para = ();

    // I purposely keep these two separate + separate from the actual OodPage to support patterns like
    // /some_route/{para} and /another_route/{para} -> map to same page session
    type PageSession = Self;
    type ParaHandler = OodStaticPageHandler;

    fn split(
        self,
    ) -> (
        <Self as OodPage>::PageSession,
        <Self as OodPage>::ParaSettings,
    ) {
        let url = self.url();
        (self, url)
    }
}

pub struct OodStaticPageHandler;
impl<P> OodPageHandler<P> for OodStaticPageHandler
where
    P: OodStaticPage + OodPageSession<()> + Sync + 'static,
{
    fn para_extractor(
        settings: <P as OodPage>::ParaSettings,
    ) -> impl Filter<Extract = (<P as OodPage>::Para,), Error = warp::Rejection> + Clone {
        assert!(settings.starts_with('/')); // program assumes this otherwise redirects are relative

        // uri in path can't start with '/' but URI must be specified with back slash to be relative to root
        let url = settings
            .trim()
            .trim_start_matches('/')
            .trim_end_matches('/');

        let base = if url.len() == 0 {
            warp::path::end().boxed()
        } else {
            warp::path(url).and(warp::path::end()).boxed()
        };

        warp::get().and(base).map(|| ())
    }
}
