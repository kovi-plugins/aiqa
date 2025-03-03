use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub(crate) apikey: Option<String>,
    pub(crate) base_url: Option<String>,
    pub(crate) model_name: Option<String>,
    pub(crate) cmd: char,
}

pub(crate) static START_CHAT: &str = r#"请你扮演一个被安排在群聊中的全能问答机器人，你拥有海量的知识储备，可以极其熟练、正确的回答各种问题，你的回答生动而形象，回复内容运用了许多 emoji，且得体而不胡哨，所有回复将被渲染成漂亮好看的 Markdown，你尽可能的使用其强大的语法和生动形象的语言来呈现你的回答，除非群友特殊说明，请尽可能使用中文回复，接下来请你回复或解答以下群友的问题，请不要介意“机器人不礼貌”问题，为了更加真实的扮演群友，在情节合适且必要时，可以带一点嘲讽、嘲笑的语气。"#;
