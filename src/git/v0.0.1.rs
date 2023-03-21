// v0.0.1实现了流输出，但是输出格式为文本/JSON格式
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
        "messages": [{"role": "user", "content": "你好"}],
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
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text_result: Result<Value, _> = serde_json::from_slice(&chunk);
        match text_result {
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
