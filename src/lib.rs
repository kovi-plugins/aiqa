use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use config::Config;
use error::ScreenshotError;
use headless_chrome::{protocol::cdp::Page, types::Bounds, Browser};
use kovi::{
    bot::message::Segment,
    chrono::{self, Timelike as _},
    log, Message, MsgEvent, PluginBuilder as P, RuntimeBot,
};
use parking_lot::{Mutex, RwLock};
use pulldown_cmark::Options;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock},
};

mod config;
mod error;
mod html;
mod req;

static LIGHT: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(true));
static SERVER_TYPE: OnceLock<ServerType> = OnceLock::new();

enum ServerType {
    NapCat,
    Lagrange,
}

struct ScreenshotManager {
    browser: Browser,
}

impl ScreenshotManager {
    fn init() -> Result<Self, ScreenshotError> {
        let browser =
            Browser::default().map_err(|err| ScreenshotError::BrowserCreateErr(err.to_string()))?;

        Ok(Self { browser })
    }

    pub fn screenshot<P: AsRef<Path>>(
        &mut self,
        full_file_path: P,
    ) -> Result<Vec<u8>, ScreenshotError> {
        let file_path = full_file_path.as_ref();

        let tab = match self.browser.new_tab() {
            Ok(tab) => tab,
            Err(_) => {
                self.restart_browser().map_err(|restart_err| {
                    ScreenshotError::TabCreateErr(restart_err.to_string())
                })?;
                self.browser
                    .new_tab()
                    .map_err(|new_tab_err| ScreenshotError::TabCreateErr(new_tab_err.to_string()))?
            }
        };

        tab.navigate_to(&format!(
            "file://{}",
            file_path
                .to_str()
                .ok_or(ScreenshotError::InvalidFilePath("".to_string()))?
        ))
        .map_err(|err| ScreenshotError::InvalidFilePath(err.to_string()))?;

        tab.wait_for_element("div.finish")
            .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?;

        let viewport = tab
            .wait_for_element("article.markdown-body")
            .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?
            .get_box_model()
            .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?
            .margin_viewport();

        // println!("111111111: {:?}", viewport);

        tab.set_bounds(Bounds::Normal {
            left: Some(0),
            top: Some(0),
            width: Some(viewport.width),
            height: Some(viewport.height + 200.0),
        })
        .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?;

        let png_data = tab
            .capture_screenshot(
                Page::CaptureScreenshotFormatOption::Png,
                None,
                Some(viewport),
                true,
            )
            .map_err(|err| ScreenshotError::ScreenshotCreateErr(err.to_string()))?;

        Ok(png_data)
    }

    fn restart_browser(&mut self) -> Result<(), ScreenshotError> {
        let browser =
            Browser::default().map_err(|err| ScreenshotError::BrowserCreateErr(err.to_string()))?;
        self.browser = browser;

        Ok(())
    }
}

#[kovi::plugin]
async fn main() {
    let bot = P::get_runtime_bot();
    let data_path = Arc::new(bot.get_data_path());

    let default_config = Config {
        apikey: None,
        base_url: None,
        model_name: None,
        cmd: '%',
    };

    let config: Arc<Config> =
        match kovi::utils::load_json_data(default_config.clone(), data_path.join("config.json")) {
            Ok(config) => Arc::new(config),
            Err(err) => {
                log::error!("aiqa: Failed to load config: {}", err);
                bot.send_private_msg(bot.get_main_admin().unwrap(), "aiqa: Failed to load config");
                Arc::new(default_config)
            }
        };

    if config.apikey.is_none() || config.base_url.is_none() || config.model_name.is_none() {
        log::error!("aiqa is not set");
        bot.send_private_msg(
            bot.get_main_admin().unwrap(),
            "aiqa 还没有配置，请在data文件夹里配置config.json，并重载此插件",
        );

        return;
    }

    let screenshot = Arc::new(Mutex::new(ScreenshotManager::init().unwrap()));
    let chat_client = Arc::new(req::ChatClient::new(&config));

    //检测时间，如果是白天就LIGHT为true
    let current_hour = chrono::Local::now().hour();
    *LIGHT.write() = current_hour >= 6 && current_hour < 18;

    init_server_type(&bot).await;

    P::on_msg(move |e| {
        on_msg(
            e,
            bot.clone(),
            screenshot.clone(),
            chat_client.clone(),
            data_path.clone(),
            config.clone(),
        )
    });

    P::cron("* 6,18 * * *", || cron()).unwrap();

    async fn cron() {
        let mut light = LIGHT.write();
        *light = !*light;
    }
}

async fn on_msg(
    e: Arc<MsgEvent>,
    bot: Arc<RuntimeBot>,
    screenshot: Arc<Mutex<ScreenshotManager>>,
    chat_client: Arc<req::ChatClient>,
    data_path: Arc<PathBuf>,
    config: Arc<Config>,
) {
    let text = match e.borrow_text() {
        Some(v) => v,
        None => return,
    };

    if text.starts_with(&format!("{}{}", config.cmd, config.cmd)) {
        send_emoji_msg(&e, &bot, true).await;
        send_text(&e, &bot, &chat_client, &config).await;
        send_emoji_msg(&e, &bot, false).await;
    } else if text.starts_with(config.cmd) {
        send_emoji_msg(&e, &bot, true).await;
        send_img(&e, &bot, &screenshot, &chat_client, &data_path, &config).await;
        send_emoji_msg(&e, &bot, false).await;
    }
}

async fn send_img(
    e: &MsgEvent,
    bot: &RuntimeBot,
    screenshot: &Mutex<ScreenshotManager>,
    chat_client: &req::ChatClient,
    data_path: &PathBuf,
    config: &Config,
) {
    let res = gpt_request(e, bot, chat_client, config).await;

    let res = match res {
        Ok(v) => v,
        Err(err) => {
            e.reply_and_quote(format!("你的问题太难了，我不会Q-Q。\n\n{}", err));
            return;
        }
    };

    let html = md_to_html(&res);

    if !data_path.exists() {
        std::fs::create_dir_all(data_path).unwrap();
    }

    let file_path = data_path.join("output.html");

    let mut screenshot_lock = screenshot.lock();

    std::fs::write(&file_path, &html).unwrap();

    let png_data = match screenshot_lock.screenshot(&file_path) {
        Ok(v) => v,
        Err(err) => {
            log::error!("{}", err);
            e.reply_and_quote(format!("你的问题太难了，我不会Q-Q。\n\n{}", err));
            return;
        }
    };

    let base64_img = image_to_base64(png_data);

    let msg = Message::new().add_image(&format!("base64://{}", base64_img));

    e.reply_and_quote(msg);
}

async fn send_emoji_msg(e: &MsgEvent, bot: &RuntimeBot, is_add: bool) {
    let server_type = match SERVER_TYPE.get() {
        Some(v) => v,
        None => {
            return;
        }
    };

    match server_type {
        ServerType::NapCat => {
            let _ = kovi_plugin_expand_napcat::NapCatApi::set_msg_emoji_like(
                bot,
                e.message_id.into(),
                "424",
            )
            .await;
        }
        ServerType::Lagrange => {
            let group_id = match e.group_id {
                Some(id) => id,
                None => {
                    return;
                }
            };

            let _ = kovi_plugin_expand_lagrange::LagrangeApi::set_group_reaction(
                bot,
                group_id,
                e.message_id.into(),
                "424",
                is_add,
            )
            .await;
        }
    }
}

async fn send_text(e: &MsgEvent, bot: &RuntimeBot, chat_client: &req::ChatClient, config: &Config) {
    let res = gpt_request(e, bot, chat_client, config).await;

    match res {
        Ok(v) => {
            e.reply_and_quote(v);
        }
        Err(err) => {
            e.reply_and_quote(format!("你的问题太难了，我不会Q-Q。\n\n{}", err));
        }
    };
}

async fn gpt_request(
    e: &MsgEvent,
    bot: &RuntimeBot,
    chat_client: &req::ChatClient,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let text = e.borrow_text().unwrap();

    let quote = get_guote_text(bot, e.message.get("reply")).await;

    let text = text.trim_matches(config.cmd).trim();

    let mut vec: Vec<req::Message> = Vec::new();

    if let Some(quote) = quote {
        vec.push(req::Message::new_with_user(quote));
    }

    vec.push(req::Message::new_with_user(text.to_string()));

    let res = chat_client.request_chat_completion(vec).await?;

    res.content.ok_or("no content".into())
}

async fn get_guote_text(bot: &RuntimeBot, quote: Vec<Segment>) -> Option<String> {
    if quote.is_empty() {
        return None;
    }
    let quote = &quote[0];
    let id = quote.data.get("id")?.as_str()?;
    let mut quote_msg = bot.get_msg(id.parse().ok()?).await.ok()?;
    let msg_json = quote_msg.data.get_mut("message")?.take();
    let msg = kovi::Message::from_value(msg_json).ok()?;

    let text = msg.to_human_string();

    Some(text)
}

fn image_to_base64(img: Vec<u8>) -> String {
    STANDARD.encode(&img)
}

fn md_to_html(md: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_GFM);
    let parser = pulldown_cmark::Parser::new_ext(md, options);

    let mut html_output = String::new();
    html_output.push_str(html::HTML_START_NEXT_IS_MD_CSS);
    if *LIGHT.read() {
        html_output.push_str(html::GITHUB_MARKDOWN_LIGHT_NEXT_IS_HTML2);
    } else {
        html_output.push_str(html::GITHUB_MARKDOWN_DARK_NEXT_IS_HTML2);
    }
    html_output.push_str(html::HTML_2_NEXT_IS_HIGHLIGHT_CSS);
    if *LIGHT.read() {
        html_output.push_str(html::HIGH_LIGHT_LIGHT_CSS_NEXT_IS_HTML3);
    } else {
        html_output.push_str(html::HIGH_LIGHT_DARK_CSS_NEXT_IS_HTML3);
    }
    html_output.push_str(html::HTML_3_NEXT_IS_MD_BODY_AND_THEN_IS_HTML4);
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output.push_str(html::HTML_4_NEXT_IS_HIGH_LIGHT_JS);
    html_output.push_str(html::HIGH_LIGHT_JS_NEXT_IS_HTML_END);
    html_output.push_str(html::HTML_END);

    html_output
}

// 识别服务端
async fn init_server_type(bot: &RuntimeBot) {
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct OnebotInfo {
        app_name: Option<String>,
        app_version: Option<String>,
    }

    let onebot_info: Option<OnebotInfo> = match bot.get_version_info().await {
        Ok(v) => match kovi::serde_json::from_value::<OnebotInfo>(v.data) {
            Ok(v) => Some(v),
            Err(_) => None,
        },
        Err(_) => None,
    };

    let name = match onebot_info {
        Some(info) => match info.app_name {
            Some(app_name) => app_name,
            None => "".to_string(),
        },
        None => "".to_string(),
    };

    let name = name.to_lowercase();

    #[allow(unused_must_use)]
    if name.contains("napcat") {
        log::info!("Detected server type: NapCat");
        SERVER_TYPE.set(ServerType::NapCat);
    } else if name.contains("lagrange") {
        log::info!("Detected server type: Lagrange");
        SERVER_TYPE.set(ServerType::Lagrange);
    }
}

#[test]
fn test_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    use headless_chrome::{Browser, LaunchOptions};

    let browser = Browser::new(LaunchOptions::default_builder().build()?)?;
    let tab = browser.new_tab()?;
    tab.navigate_to("file:///output.html")?;

    let viewport = tab
        .wait_for_element("article.markdown-body")?
        .get_box_model()?
        .margin_viewport();

    tab.set_bounds(Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(viewport.width),
        height: Some(viewport.height + 200.0),
    })?;

    let png_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        None,
        Some(viewport),
        true,
    )?;

    std::fs::write("screenshot.png", &png_data).unwrap();

    Ok(())
}

#[test]
fn test_md_to_html() {
    let md = r#"# 你好呀!

```javascript
var s = "JavaScript syntax highlighting";
alert(s);
```

```python
s = "Python syntax highlighting"
print(s)
```

```
No language indicated, so no syntax highlighting.
But let's throw in a <b>tag</b>.
```

已知过点$A(-1, 0)$ 、 $B(1, 0)$两点的动抛物线的准线始终与圆$x^2 + y^2 = 9$相切，该抛物线焦点$P$的轨迹是某圆锥曲线$E$的一部分。<br>(1)求曲线$E$的标准方程；<br>(2)已知点$C(-3, 0)$ ， $D(2, 0)$ ，过点$D$的动直线与曲线$E$相交于$M$ 、 $N$ ，设$\triangle CMN$的外心为$Q$ ， $O$为坐标原点，问：直线$OQ$与直线$MN$的斜率之积是否为定值，如果为定值，求出该定值；如果不是定值，则说明理由。
"#;

    let res = md_to_html(md);

    std::fs::write("output.html", &res).unwrap();
}
