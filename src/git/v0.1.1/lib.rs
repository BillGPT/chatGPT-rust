// src/lib.rs
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::io::{self, Write};

pub async fn get_response(
    api_key: &str,
    api_url: &str,
    content: &str,
) -> Result<(), reqwest::Error> {
    let payload = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": content}],
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
    let mut result_text = String::new();

    while let Some(chunk) = stream.next().await {
        let mut output = String::new();

        let chunk = chunk?;
        let mut utf8_str = String::from_utf8_lossy(&chunk);
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
                                let content_str = content_str.trim_start_matches('\n');
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
                    write!(stdout_lock, "{}", c).unwrap();
                    stdout_lock.flush().unwrap();
                }
            }
            Err(e) => {}
        }
    }
    print!("\n");
    println!("------");
    println!("{:?}", result_text);
    println!("------");
    Ok(())
}

pub fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut content = String::new();
    io::stdin()
        .read_line(&mut content)
        .expect("Error reading input");
    content.trim().to_string()
}
