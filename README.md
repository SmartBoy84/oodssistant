# Rust library to interface with the Bark notify app
By far my most complex project yet. It includes two API libraries (Gcal -> OAuth included, and bark notification API) implemented using my restman-rs library, a complex asynchronous iOS shortcuts web "interface" with sessions and internal redirects and a manager to top it all off. It's essentially a glorified todo list/time manager.  

# Next steps
- Figure out how to transfer state data in redirects (e.g., Homepage -> subpage)
- Figure out OneshotPages (do not generate a new session BUT the handler can only run b.cf ONCE)
- Probably more but I am brain dead righ tnow


- Test one-shot, multi-shot
    - [x] Firstly, is basic page functionality working?
        - I.e., all the ood actions (button, text input, timer, external url)  
    - [x] Can you access external state?
    - [ ] ~~Is one-shot working?~~ Abandoned, don't need it + way too complex
    - [x] Is redirect working? 
    - [x] dynamic pages (e.g., pages with same handler but different URLs), 
    - [ ] custom query parameters (e.g., page wiht custom handler), 
    - [ ] static page
    - [ ] pages only accessible through another page
    - [ ] Test same page but parameter obtained differently (disconnect between OodSession and ParaHandler)
    - [ ] test redirect cache persistence

# Examples
```rust
    let server = OodServerBuilder::new(SocketAddr::from_str(OOD_SERVER_URI).unwrap())
        .add_route(OodStatic(Homepage::new(OOD_SHORTCUT_NAME)))
        .add_route(OodStatic(DynamicStaticTest("/a")))
        .add_route(OodStatic(DynamicStaticTest("/b")))
        .add_route(OodPara(ParaPageTest))
        .start_server()
        .await_server()
        .await
        .unwrap();
```

```rust
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
    const URI: &str = "";
}

impl OodPageSession<()> for Homepage {
    async fn start_session(self, mut b: OodBridge, _: ()) -> Result<OodFinished, OodAppErr> {
        let Self { shortcut_name } = self;
        println!("New homepage!");

        let res = b.cf(&OodTextInput::new("where to?", "")).await?;

        let url = &OodShortcut::new(shortcut_name, res).to_string();
        b.cf(&OodInfo::new("Going to:", &url)).await?;

        b.cf(&OodOpenUri::new("", &url)).await?;

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
    const URI: &'static str = "name";
    type Para = String;
}

impl OodPageSession<String> for ParaPageTest {
    async fn start_session(self, mut b: OodBridge, p: String) -> Result<OodFinished, OodAppErr> {
        b.cf(&OodInfo::new("Hello", &format!("Hey there, {p}!")))
            .await?;
        Ok(b.finished().await)
    }
}
```