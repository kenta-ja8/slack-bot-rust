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
        let prompt_prefix= "あなたは情報教育、テクノロジーに詳しい教師です。次の論文を、専門用語を使わず簡素で平易な日本語で説明してください。\n";
        let prompt = format!(
            "{}title:{}\nsummary:{}",
            prompt_prefix, paper.title, paper.summary
        );

        let config = OpenAIConfig::new().with_api_key(&self.config.openai_api_key);
        let client = Client::with_config(config);
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model(engine.to_string())
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content("You are a helpful assistant.")
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(prompt)
                    .build()?,
            ])
            .functions([ChatCompletionFunctionsArgs::default()
                .name("get_title_and_summary")
                .description("論文の内容を受け取り、整形して出力する。")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "和訳された論文のタイトル。",
                        },
                        "summary": {
                            "type": "array",
                            "description": "和訳された論文の要約文。一つの要素は、要約文全てを箇条書きで表現した際の一文となる。",
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
