use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use config::Config;
use kovi::chrono::{self, Timelike as _};
use kovi::event::MessageEventTrait;
use kovi::{Message, PluginBuilder as P, RuntimeBot, Segment as KoviSegment, log};
use parking_lot::{Mutex, RwLock};
use pulldown_cmark::Options;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

#[cfg(not(any(feature = "napcat-onebot", feature = "milky")))]
compile_error!("请至少启用一个协议 feature: \"napcat-onebot\" 或 \"milky\"");

#[cfg(all(feature = "napcat-onebot", feature = "milky"))]
compile_error!("不能同时启用 napcat-onebot 和 milky feature");

#[cfg(feature = "napcat-onebot")]
use kovi_onebot::*;

#[cfg(feature = "milky")]
use kovi_milky::*;

#[cfg(any(feature = "napcat-onebot", feature = "milky"))]
use crate::browser::ScreenshotManager;

mod browser;
#[cfg(any(feature = "napcat-onebot", feature = "milky"))]
mod config;
mod error;
mod html;
mod req;

static LIGHT: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(true));

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

    let (config, send_err_msg) = {
        let fallback = default_config.clone();
        match kovi::utils::load_json_data(default_config, data_path.join("config.json")) {
            Ok(config) => (Arc::new(config), None),
            Err(err) => {
                log::error!("aiqa: Failed to load config: {}", err);
                (Arc::new(fallback), Some("aiqa: Failed to load config"))
            }
        }
    };
    if let Some(msg) = send_err_msg {
        send_private_msg(
            &bot,
            bot.get_main_admin().unwrap().try_as_i64_or_panic(),
            msg,
        )
        .await;
    }

    if config.apikey.is_none() || config.base_url.is_none() || config.model_name.is_none() {
        log::error!("aiqa is not set");
        send_private_msg(
            &bot,
            bot.get_main_admin().unwrap().try_as_i64().unwrap(),
            "aiqa 还没有配置，请在data文件夹里配置config.json，并重载此插件",
        )
        .await;

        return;
    }

    let screenshot = Arc::new(Mutex::new(ScreenshotManager::init().unwrap()));
    let chat_client = Arc::new(req::ChatClient::new(&config));

    //检测时间，如果是白天就LIGHT为true
    let current_hour = chrono::Local::now().hour();
    *LIGHT.write() = current_hour >= 6 && current_hour < 18;

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

    P::cron("0 6,18 * * *", || cron()).unwrap();

    async fn cron() {
        let mut light = LIGHT.write();
        *light = !*light;
    }
}

#[cfg(any(feature = "napcat-onebot", feature = "milky"))]
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

#[cfg(any(feature = "napcat-onebot", feature = "milky"))]
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

#[cfg(feature = "napcat-onebot")]
async fn send_emoji_msg(e: &MsgEvent, bot: &RuntimeBot, _is_add: bool) {
    let _ =
        kovi_plugin_expand_napcat::NapCatApi::set_msg_emoji_like(bot, e.message_id.into(), "424")
            .await;
}

#[cfg(feature = "milky")]
async fn send_emoji_msg(e: &MsgEvent, bot: &RuntimeBot, is_add: bool) {
    use kovi_milky::MilkyGroupApi;

    let group_id = match e.data.group.as_ref() {
        Some(group) => group.group_id,
        None => return,
    };

    bot.send_group_message_reaction(group_id, e.data.message_seq, "424", "face", is_add);
}

#[cfg(feature = "napcat-onebot")]
async fn send_private_msg(bot: &RuntimeBot, user_id: i64, text: &str) {
    bot.send_private_msg(user_id, text);
}

#[cfg(feature = "milky")]
async fn send_private_msg(bot: &RuntimeBot, user_id: i64, text: &str) {
    use kovi_milky::MilkyMessageApi;
    let msg = Message::new().add_text(text);
    let _ = bot.send_private_message(user_id, msg).await;
}

#[cfg(any(feature = "napcat-onebot", feature = "milky"))]
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

#[cfg(any(feature = "napcat-onebot", feature = "milky"))]
async fn gpt_request(
    e: &MsgEvent,
    bot: &RuntimeBot,
    chat_client: &req::ChatClient,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let text = e.borrow_text().unwrap();

    let quote = get_guote_text(bot, e, e.get_message().get("reply")).await;

    let text = text.trim_matches(config.cmd).trim();

    let mut vec: Vec<req::Message> = Vec::new();

    if let Some(quote) = quote {
        vec.push(req::Message::new_with_user(quote));
    }

    vec.push(req::Message::new_with_user(text.to_string()));

    let res = chat_client.request_chat_completion(vec).await?;

    res.content.ok_or("no content".into())
}

#[cfg(feature = "napcat-onebot")]
async fn get_guote_text(
    bot: &RuntimeBot,
    _e: &MsgEvent,
    quote: Vec<KoviSegment>,
) -> Option<String> {
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

#[cfg(feature = "milky")]
async fn get_guote_text(bot: &RuntimeBot, e: &MsgEvent, quote: Vec<KoviSegment>) -> Option<String> {
    use kovi_milky::MilkyMessageApi;

    if quote.is_empty() {
        return None;
    }
    let quote = &quote[0];
    let message_seq: i64 = quote.data.get("message_seq")?.as_i64()?;

    let group_id = e.data.group.as_ref()?.group_id;

    let mut quote_msg = bot.get_message("group", group_id, message_seq).await.ok()?;
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

#[test]
#[ignore = "需要本地文件"]
fn test_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    use headless_chrome::{Browser, LaunchOptions};

    let browser = Browser::new(LaunchOptions::default_builder().build()?)?;
    let tab = browser.new_tab()?;
    tab.navigate_to("file:///output.html")?;

    let viewport = tab
        .wait_for_element("article.markdown-body")?
        .get_box_model()?
        .margin_viewport();

    tab.set_bounds(headless_chrome::types::Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(viewport.width),
        height: Some(viewport.height + 200.0),
    })?;

    // 设置设备像素比
    tab.call_method(
        headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride {
            width: viewport.width as u32,
            height: (viewport.height + 200.0) as u32,
            device_scale_factor: 2.0,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        },
    )
    .map_err(|err| err.to_string())?;

    let png_data = tab.capture_screenshot(
        headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
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
