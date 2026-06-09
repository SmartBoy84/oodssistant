use warp::Filter;

use crate::server::interface::page::{OodPage, OodPageHandler, OodPageSession};

pub struct OodStatic<T: OodStaticPage>(pub T);

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

impl<P> OodPage for OodStatic<P>
where
    P: OodStaticPage + Send + Sync + 'static,
{
    // here's the trick to simplify it for the end user - Self is the page session!
    type ParaSettings = &'static str;
    type Para = ();

    // I purposely keep these two separate + separate from the actual OodPage to support patterns like
    // /some_route/{para} and /another_route/{para} -> map to same page session
    type PageSession = P;
    type ParaHandler = OodStaticPageHandler;

    fn split(
        self,
    ) -> (
        <Self as OodPage>::PageSession,
        <Self as OodPage>::ParaSettings,
    ) {
        let url = self.0.url();
        (self.0, url)
    }
}

// keep OodPageHandler separate
pub struct OodStaticPageHandler;
impl OodPageHandler<&'static str, ()> for OodStaticPageHandler {
    fn para_extractor(
        settings: &'static str,
    ) -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone + Send {
        if !settings.starts_with('/') {
            // program assumes this otherwise redirects are relative
            panic!("'{settings}' doesn't start with '/'!");
        }
        // uri in path can't start with '/' but URI must be specified with back slash to be relative to root
        let url = settings
            .trim()
            .trim_start_matches('/')
            .trim_end_matches('/');

        let mut path = warp::get().boxed();
        if url.len() > 0 {
            path = path.and(warp::path(url)).boxed();
        };

        path.map(|| ())
    }
}
