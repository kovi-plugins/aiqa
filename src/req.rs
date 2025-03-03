use crate::*;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionResponseMessage,
        CreateChatCompletionRequestArgs, ResponseFormat,
    },
    Client,
};
use config::START_CHAT;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    role: Role,
    content: String,
}

impl Message {
    pub fn new(role: Role, content: String) -> Message {
        Message { role, content }
    }

    pub fn new_with_user(content: String) -> Message {
        Message::new(Role::User, content)
    }
}

pub struct ChatClient {
    client: Client<OpenAIConfig>,
    model_name: String,
}

impl ChatClient {
    pub fn new(config_: &Config) -> ChatClient {
        let config = OpenAIConfig::new()
            .with_api_base(config_.base_url.clone().unwrap())
            .with_api_key(config_.apikey.clone().unwrap());

        ChatClient {
            client: async_openai::Client::with_config(config),
            model_name: config_.model_name.clone().unwrap(),
        }
    }

    pub async fn request_chat_completion(
        &self,
        msgs: Vec<Message>,
    ) -> Result<ChatCompletionResponseMessage, Box<dyn Error>> {
        let mut send_msgs = Vec::with_capacity(msgs.len() + 1);

        send_msgs.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(START_CHAT)
                .build()
                .unwrap()
                .into(),
        );

        for msg in msgs.iter() {
            match msg.role {
                Role::System => {
                    send_msgs.push(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(msg.content.clone())
                            .build()
                            .unwrap()
                            .into(),
                    );
                }
                Role::User => {
                    send_msgs.push(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(msg.content.clone())
                            .build()
                            .unwrap()
                            .into(),
                    );
                }
                Role::Assistant => {
                    send_msgs.push(
                        ChatCompletionRequestAssistantMessageArgs::default()
                            .content(msg.content.clone())
                            .build()
                            .unwrap()
                            .into(),
                    );
                }
            }
        }

        let request = CreateChatCompletionRequestArgs::default()
            // .max_tokens(MOBEL_MAX_TOKEN)
            .model(self.model_name.clone())
            .messages(send_msgs)
            .response_format(ResponseFormat::Text)
            .build()
            .unwrap();

        let choice = {
            let mut response = self.client.chat().create(request).await?;
            response.choices.pop()
        };

        match choice {
            Some(v) => Ok(v.message),
            None => Err("请求失败".into()),
        }
    }
}

// pub(crate) async fn request_chat_completion(
//     messages: Vec<AiMessage>,
// ) -> Result<ApiResponse, Box<dyn Error>> {
//     let timeout_duration = Duration::from_secs(30);

//     let client = Client::builder().timeout(timeout_duration).build()?;

//     let chat_request = ChatRequest {
//         model: "glm-4-plus".to_string(),
//         messages,
//     };

//     let response_result = timeout(timeout_duration, async {
//         let response = client
//             .post("https://open.bigmodel.cn/api/paas/v4/chat/completions")
//             .header("Authorization", format!("Bearer {}", APIKEY))
//             .header("Content-Type", "application/json")
//             .json(&chat_request)
//             .send()
//             .await?;

//         let chat_response = response.json::<ApiResponse>().await?;

//         Ok(chat_response)
//     })
//     .await;

//     // 检查是否超时
//     match response_result {
//         Ok(result) => result,
//         Err(_) => Err("请求超时".into()),
//     }
// }

// #[tokio::test]
// async fn ai_chat() {
//     let config = crate::config::Config

//     let client = ChatClient::new();

//     let msgs = vec![Message {
//         role: Role::User,
//         content: "我有一个群聊机器人的一个插件，可以请求ai进行回答，曾经我命名为“gpt”，现在我希望可以让这个名字更加通用，有什么推荐的名字吗".to_string(),
//     }];

//     let response = client.request_chat_completion(msgs).await.unwrap();

//     println!("{:?}", response);
// }
