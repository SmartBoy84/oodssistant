# Rust library to interface with the Bark notify app
By far my most complex project yet. It includes two API libraries (Gcal -> OAuth included, and bark notification API) implemented using my restman-rs library, a complex asynchronous iOS shortcuts web "interface" with sessions and internal redirects and a manager to top it all off. It's essentially a glorified todo list/time manager.  

# Next steps
- Figure out how to transfer state data in redirects (e.g., Homepage -> subpage)
- Figure out OneshotPages (do not generate a new session BUT the handler can only run b.cf ONCE)
- Probably more but I am brain dead righ tnow


- Test one-shot, multi-shot
    - [x] Firstly, is basic page functionality working?
        - [x] I.e., all the ood actions (button, text input, timer, external url, internal redirect, external redirect)  
    - [x] Can you access external state?
    - [x] ~~Is one-shot working?~~ Abandoned, don't need it + way too complex
    - [x] Is redirect working? 
    - [x] dynamic pages (e.g., pages with same handler but different URLs), 
    - [x] custom query parameters (e.g., page wiht custom handler), 
    - [x] pages only accessible through another page
    - [x] test redirect cache persistence

# Examples
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
            "Hidden page, with session",
            "Cool side effect",
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
        } else if res == buttons[3] {
            return Ok(b
                .redirect(
                    HiddenPage,
                    HiddenPara {
                        name: "hamdan (hidden)".to_string(),
                        age: 20,
                    },
                )
                .await);
        } else if res == buttons[4] {
            b.cf(&OodOpenUri::new(
                "",
                &OodShortcut::new(shortcut_name, CoolSideEffect::URI).to_string(),
            ))
            .await?;
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

#[derive(Debug)]
struct HiddenPara {
    name: String,
    age: usize,
}
impl OodPagePara for HiddenPara {}

#[derive(Clone)]
pub struct HiddenPage;
impl OodPageSession<HiddenPara> for HiddenPage {
    async fn start_session(
        self,
        mut b: OodBridge,
        p: HiddenPara,
    ) -> Result<OodFinished, OodAppErr> {
        let HiddenPara { name, age } = p;
        b.cf(&OodInfo::new("Yo!", "Ima gonna show sum bits bout u init"))
            .await?;
        b.cf(&OodInfo::new(
            "Hey man!",
            &format!("You are {name}\nYou are {age} years old!"),
        ))
        .await?;

        for i in 0..10 {
            b.cf(&OodInfo::new(&format!("{i}"), "")).await?;
        }

        Ok(b.finished().await)
    }
}

#[derive(Clone)]
pub struct CoolSideEffect {
    pub s: Arc<Mutex<Option<String>>>, // any Arc will be part of every session!!
}

impl OodBasicPage for CoolSideEffect {
    const URI: &str = "/cool";
}

impl OodPageSession<()> for CoolSideEffect {
    async fn start_session(self, mut b: OodBridge, _: ()) -> Result<OodFinished, OodAppErr> {
        println!("new!");
        let Self { s } = self;
        let res = b.cf(&OodTextInput::new("Essay", "")).await?;
        let mut lock = s.lock().await;
        match &mut *lock {
            Some(s) => {
                s.push_str(&res);
                b.cf(&OodInfo::new("Updated essay", s.as_str())).await?;
            }
            None => {
                b.cf(&OodInfo::new("New essay", res.as_str())).await?;
                *lock = Some(res);
            }
        }
        Ok(b.finished().await)
    }
}
```

# Cache persistence example
```rust
#[derive(Clone)]
pub struct Homepage {
    cache: Arc<Mutex<Vec<(String, SessionId)>>>,
}

impl Homepage {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl OodBasicPage for Homepage {
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    type SessionPara = OodSessionPara;
    async fn start_session(
        self,
        mut b: OodBridge,
        _: (),
        sp: OodSessionPara,
    ) -> Result<OodFinished, OodAppErr> {
        println!("New homepage!");

        loop {
            match b
                .cf(&OodButtonList::new(
                    "",
                    &["New session", "Restore old session"],
                ))
                .await?
                .as_str()
            {
                "New session" => {
                    let name = b.cf(&OodTextInput::new("Session name", "")).await?;
                    self.cache.lock().await.push((name, sp.session_id));
                    for i in 0..100 {
                        b.cf(&OodInfo::new("", &i.to_string())).await?;
                    }
                    break;
                }
                "Restore old session" => {
                    if self.cache.lock().await.len() == 0 {
                        b.cf(&OodInfo::new("Oops!", "No previous session")).await?;
                    } else {
                        let names = self
                            .cache
                            .lock()
                            .await
                            .iter()
                            .map(|(n, _)| n.to_string())
                            .collect::<Vec<String>>();
                        println!("Sessions: {names:?}");
                        let res = b
                            .cf(&OodButtonList::new("Select session", names.as_slice()))
                            .await?;
                        let s_id = self
                            .cache
                            .lock()
                            .await
                            .iter()
                            .find_map(|(n, s_id)| (n == &res).then_some(s_id))
                            .unwrap()
                            .to_owned();
                        return Ok(b.external_redirect(s_id.clone()).await);
                    }
                }
                _ => unreachable!(),
            };
        }
        Ok(b.finished().await)
    }
}
```