use futures_util::stream::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").unwrap();
    let api_url = "https://api.openai.com/v1/chat/completions";

    let payload = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": "You can only answer: Okey"}],
        "temperature": 1.0,
        "top_p": 1.0,
        "n": 1,
        "stream": true,
        "presence_penalty": 0,
        "frequency_penalty": 0
    });

    let client = Client::new();
    let response = client
        .post(api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut i = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let mut utf8_str = String::from_utf8_lossy(&chunk).to_string();
        println!("---");
        println!("{}", utf8_str);

        if i == 0 {
            // TODO：修改utf8_str的值为utf8_str本身的倒数第二行的值。
            let lines: Vec<&str> = utf8_str.lines().collect();
            let updated_utf8_str = if lines.len() >= 2 {
                lines[lines.len() - 2].to_string()
            } else {
                utf8_str.clone()
            };
            utf8_str = updated_utf8_str;
            i += 1;
        }

        let trimmed_str = utf8_str.trim_start_matches("data: ");
        println!("{}", trimmed_str);

        let json_result: Result<Value, _> = serde_json::from_str(trimmed_str);
        match json_result {
            Ok(text) => {
                println!("{:?}", text);
            }
            Err(_) => {
                eprintln!("Error parsing JSON: {}", String::from_utf8_lossy(&chunk));
            }
        }
    }

    Ok(())
}
