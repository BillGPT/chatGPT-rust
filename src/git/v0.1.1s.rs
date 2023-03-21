// v0.1.1s实现保存历史对话的功能，还没有实现余弦相似度的计算
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::io::prelude::*;
use std::io::stdin;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use text_io::read;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

async fn save_embeddings_to_json(
    json_data: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = json_data.to_string();
    let file_path = format!("log/{}.json", Uuid::new_v4().to_string());
    let mut file = fs::File::create(file_path).await?;
    file.write_all(json_string.as_bytes()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let api_url = "https://api.openai.com/v1/chat/completions";
    let api_url_embedding = "https://api.openai.com/v1/embeddings";

    print!("初始化：");
    io::stdout().flush()?;
    let mut content = String::new();
    stdin()
        .read_line(&mut content)
        .expect("Error reading input");
    let content = content.trim().to_string(); // 移除行尾的换行符

    let mut i = 0;
    let mut result_text = String::new();

    loop {
        // 用户输入content内容
        if i != 0 {
            //println!("{}", result_text);
            if i % 2 != 0 {
                print!("🧑‍💻：");
                i += 1;
            } else {
                print!("🦾🤖：");
                i += 1;
            }
        } else {
            result_text = content.clone();
            print!("🧑‍💻：");
            println!("{}", result_text);
            print!("🦾🤖：");
            i += 1;
        }

        let payload = serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "你是一个土生土长的古代中国人，要用文言文，字字珠玑，话要短，要玄乎"},
                {"role": "user", "content": content}],
            "temperature": 1.0,
            "top_p": 1.0,
            "n": 1,
            "max_tokens": 256,
            "stream": true,
            "presence_penalty": 2,
            "frequency_penalty": 2
        });

        let payload_embedding = serde_json::json!({
            "model": "text-embedding-ada-002",
            "input": result_text
        });

        let client = Client::new();
        let response = client
            .post(api_url_embedding)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload_embedding)
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("{}", response_text);
        let json_data: serde_json::Value = serde_json::from_str(&response_text)?;
        save_embeddings_to_json(json_data).await?;

        result_text = String::new();

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
            let mut output = String::new();

            let chunk = chunk?;
            let mut utf8_str = String::from_utf8_lossy(&chunk).to_string();
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
            let json_result: Result<Value, _> = serde_json::from_str(trimmed_str);
            match json_result {
                Ok(json) => {
                    if let Some(choices) = json.get("choices") {
                        if let Some(choice) = choices.get(0) {
                            if let Some(content) =
                                choice.get("delta").and_then(|delta| delta.get("content"))
                            {
                                if let Some(content_str) = content.as_str() {
                                    //let content_str = content_str.trim_start_matches('\n');
                                    output.push_str(content_str);
                                    result_text.push_str(content_str);
                                } else {
                                }
                            } else {
                            }
                        } else {
                        }
                    } else {
                    }

                    let stdout = io::stdout();
                    let mut stdout_lock = stdout.lock();
                    for c in output.chars() {
                        write!(stdout_lock, "{}", c)?;
                        stdout_lock.flush()?;
                    }
                }
                Err(e) => {}
            }
        }
        print!("\n");
        println!("AAA: {}", result_text);
        thread::sleep(Duration::from_secs(1));
    }
}
