// src/main.rs
use chatgpt_wasm::{get_response, read_user_input};
use std::env;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").unwrap();
    let api_url = "https://api.openai.com/v1/chat/completions";

    loop {
        let content = read_user_input("🧑‍💻：");
        println!("------");
        println!("{}", String::from(&content));
        println!("------");
        print!("🦾🤖：");
        get_response(&api_key, api_url, &content).await?;
    }
}
