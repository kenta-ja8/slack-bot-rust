mod client;
mod model;

async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let config = model::config::load_config()?;
    let slack = client::slack::SlackClient::new(&config);
    let text = "
---Hello Slack---
• XXX
• YYY
qqq `aaa` www
```
bbb
```
---Goodby Slack---

";
    let info = format!("Running on {}", config.platform);
    let message = format!("{}{}", text, info);
    slack.post_message(message).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Start Job");

    match execute().await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("End Job");
}
