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
Do not write in English. 
"
        .trim();
        let user_prompt_prefix = "
Explain the following a paper in simple, plain, jargon-free Japanese.
The output should be specified formatted."
            .trim();
        let user_prompt = format!(
            "{}\ntitle:{}\nsummary:{}",
            user_prompt_prefix, paper.title, paper.summary
        );

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
                    .content(format!("{}{}", system_prompt, user_prompt))
                    .build()?,
            ])
            .functions([ChatCompletionFunctionsArgs::default()
                .name("convert_to_specified_format")
                .description("Convert to specified format.")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the paper written in Japanese.",
                        },
                        "summary": {
                            "type": "array",
                            "description": "Paper summary text written in Japanese. One element is a sentence when all the summary text is expressed in bullet points.",
                            "items" : {
                              "type": "string",
                            },
                        },
                    },
                    "required": ["title", "summary"],
                }))
                .build()?])
            .function_call("auto")
            .build()?;

        let response = client.chat().create(request).await?;
        for choice in response.choices {
            // println!("choice: {:#?}", choice);
            if let Some(r) = choice.finish_reason {
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
