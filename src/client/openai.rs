use anyhow::{Error, Result};
use async_openai::types::ChatCompletionFunctionsArgs;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use serde_json::json;

use crate::model::openai::PaperSummaryModel;
use crate::model::paper::PaperModel;
use crate::model::{config::Config, openai::Engine};

pub struct OpenAiClient<'a> {
    config: &'a Config,
}

impl<'a> OpenAiClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn summarize_paper(
        &self,
        paper: &PaperModel,
        engine: &Engine,
    ) -> Result<PaperSummaryModel> {
        let system_prompt = "
            You are a teacher with expertise in information education and technology.
            Explain the following a paper in simple, plain, jargon-free Japanese. Do not use English.".trim();
        let user_prompt = format!("title:{}\nsummary:{}", paper.title, paper.summary);

        let config = OpenAIConfig::new().with_api_key(&self.config.openai_api_key);
        let client = Client::with_config(config);
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model(engine.to_string())
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(system_prompt)
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(user_prompt)
                    .build()?,
            ])
            .functions([ChatCompletionFunctionsArgs::default()
                .name("get_title_and_summary")
                .description("日本語で記述されたの論文の内容を受け取り、整形して出力する。")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "日本語で記述された論文のタイトル。",
                        },
                        "summary": {
                            "type": "array",
                            "description": "日本語で記述された論文の要約文。一つの要素は、要約文全てを箇条書きで表現した際の一文となる。",
                            "items" : {
                              "type": "string",
                            },
                        },
                    },
                    "required": ["title", "summary"],
                }))
                .build()?])
            .build()?;

        let response = client.chat().create(request).await?;
        for choice in response.choices {
            // println!(
            //     "{}: Role: {}  Content: {:?}",
            //     choice.index, choice.message.role, choice.message.content
            // );
            if let Some(r) = choice.finish_reason {
                // println!("Value is: {}", r);
                if r == "function_call" {
                    if let Some(f) = choice.message.function_call {
                        let p: PaperSummaryModel = serde_json::from_str(&f.arguments)?;
                        return Ok(p);
                    }
                }
            }
        }
        Err(Error::msg("Failed to get function_call answer"))
    }
}
