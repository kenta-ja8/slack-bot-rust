use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

use crate::model::config::Config;

pub struct OpenAiClient<'a> {
    config: &'a Config,
}

impl<'a> OpenAiClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn chat(&self, text: String) -> Result<String, Box<dyn std::error::Error>> {
        let config = OpenAIConfig::new().with_api_key(&self.config.openai_api_key);

        let client = Client::with_config(config);

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model("gpt-3.5-turbo")
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content("You are a helpful assistant.")
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(text)
                    .build()?,
            ])
            .build()?;

        let response = client.chat().create(request).await?;

        let mut sentence = String::new();
        for choice in response.choices {
            sentence.push_str(&choice.message.content)
        }

        Ok(sentence)
    }
}
