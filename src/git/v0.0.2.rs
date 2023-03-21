// v0.0.2实现了流输出文字，从JSON提取出Content文字，但是还没有实现逐字输出，每次都会输出最新的一行
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
        "messages": [{"role": "user", "content": "hi"}],
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
    let mut output = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        //println!("{:?}", chunk);
        let mut utf8_str = String::from_utf8_lossy(&chunk);
        //println!("{:?}", utf8_str);

        let trimmed_str = utf8_str.trim_start_matches("data: ");
        let json_result: Result<Value, _> = serde_json::from_str(trimmed_str);
        match json_result {
            Ok(json) => {
                //println!("JSON: {:?}", json);

                if let Some(choices) = json.get("choices") {
                    if let Some(choice) = choices.get(0) {
                        if let Some(content) =
                            choice.get("delta").and_then(|delta| delta.get("content"))
                        {
                            if let Some(content_str) = content.as_str() {
                                output.push_str(content_str);
                            } else {
                                //println!("Content value is not a string");
                            }
                        } else {
                            //println!("No 'content' key found");
                        }
                    } else {
                        //println!("No choice found");
                    }
                } else {
                    //println!("No 'choices' key found");
                }

                // Replace \n with a newline character
                let formatted_output = output.replace("\\n", "\n");
                print!("{}", formatted_output);
            }
            Err(e) => {
                //eprintln!("Error parsing JSON: {}, error: {:?}", utf8_str, e);
            }
        }
    }

    Ok(())
}
