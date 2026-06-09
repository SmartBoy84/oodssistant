use crate::{
    brain::OodShortcut,
    server::interface::{
        OodAction, OodAppErr,
        bridge::{OodBridge, OodFinished},
        elements::{OodButtonList, OodInfo, OodOpenUri, OodTextInput},
        page::{
            OodPage, OodPageSession,
            basic::{OodBasicPage, OodStaticPage},
            para::OodParaPage,
        },
    },
};

#[derive(Clone)]
pub struct Homepage {
    pub shortcut_name: &'static str,
}

impl Homepage {
    pub fn new(shortcut_name: &'static str) -> Self {
        Self { shortcut_name }
    }
}

impl OodBasicPage for Homepage {
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    async fn start_session(self, mut b: OodBridge, _: ()) -> Result<OodFinished, OodAppErr> {
        let Self { shortcut_name } = self;
        println!("New homepage!");

        let buttons = [
            "Static test",
            "Dynamic test (external)",
            "Dynamic test (internal)",
        ];
        let res = b.cf(&OodButtonList::new("Index", &buttons)).await?;

        if res == buttons[0] {
            let res = b.cf(&OodTextInput::new("where to?", "")).await?;

            let url = &OodShortcut::new(shortcut_name, res).to_string();
            b.cf(&OodInfo::new("Going to:", &url)).await?;

            b.cf(&OodOpenUri::new("", &url)).await?;
        } else if res == buttons[1] {
            let uri = ParaPageTest::create_uri("HAMDAN!(external)");
            let shortcut = OodShortcut::new(shortcut_name, uri).to_string();
            b.cf(&OodInfo::new("External test:", &shortcut)).await?;
            b.cf(&OodOpenUri::new("", &shortcut)).await?;
        } else if res == buttons[2] {
            return Ok(b
                .redirect(ParaPageTest, "hamdan (internal)".to_string())
                .await);
        }

        Ok(b.finished().await)
    }
}

#[derive(Clone)]
pub struct DynamicStaticTest(pub &'static str);

impl OodStaticPage for DynamicStaticTest {
    fn url(&self) -> &'static str {
        &self.0
    }
}

impl OodPageSession<()> for DynamicStaticTest {
    async fn start_session(
        self,
        mut b: crate::server::interface::bridge::OodBridge,
        _: (),
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::OodAppErr>
    {
        b.cf(&OodInfo::new("Url", &format!("My root is at {}", self.0)))
            .await?;

        Ok(b.finished().await)
    }
}

#[derive(Clone)]
pub struct ParaPageTest;
impl OodParaPage for ParaPageTest {
    const URI: &'static str = "/name";
    type Para = String;
}

impl OodPageSession<String> for ParaPageTest {
    async fn start_session(self, mut b: OodBridge, p: String) -> Result<OodFinished, OodAppErr> {
        b.cf(&OodInfo::new("Hello", &format!("Hey there, {p}!")))
            .await?;
        Ok(b.finished().await)
    }
}
