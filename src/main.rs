mod client;
mod model;

use anyhow::Result;

async fn execute() -> Result<()> {
    let config = model::config::load_config()?;
    let arxiv = client::arxiv::ArxivClient::new(&config);
    let openapi = client::openapi::OpenAiClient::new(&config);
    let slack = client::slack::SlackClient::new(&config);

    let papers = arxiv.search_past_24_to_48_hours().await?;
    if papers.len() == 0 {
        println!("not found paper");
        return Ok(());
    }

    let prompt_prefix= "あなたは情報教育、テクノロジーに詳しい教師です。次の論文を、タイトルと要約の2点を専門用語を使わず、簡素で平易な日本語で説明してください。出力は箇条書きでお願いします。\n";
    let mut slack_msg = String::new();
    for p in papers {
        let answer = openapi
            .chat(format!(
                "{}title:{}\nsummary:{}",
                prompt_prefix, p.title, p.summary
            ))
            .await?;
        slack_msg.push_str(&format!("*<{} | {}>*\n", p.url, p.title));
        slack_msg.push_str(&format!("{}\n", &answer));
        slack_msg.push_str(
            "-------------------------------------------------------------------------------\n\n",
        );
    }
    slack_msg.push_str(&format!(
        "_Powered by ChatGPT  / Running on {}_\n",
        config.platform
    ));
    slack.post_message(slack_msg).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Start Job");

    match execute().await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            e.backtrace();
            std::process::exit(1);
        }
    };

    println!("End Job");
}
