// v0.1.0实现了循环对话交互，还没有部署到wasm/yew上
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
//use std::io::prelude::*;
use std::io::stdin;
use std::io::{self, Write};
//use std::path::Path;
//use std::thread;
//use std::time::Duration;
//use text_io::read;
use ndarray::prelude::*;
use ndarray::Array1;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

async fn summarize_memories(
    memories: &[Memory],
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_url = "https://api.openai.com/v1/chat/completions";
    let input_text = memories
        .iter()
        .map(|memory| memory.message_info.message.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    println!("input_text: {}", input_text.clone());
    let prompt = format!("请总结以下文本:\n{}", input_text);

    let payload = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "system", "content": prompt}],
        "max_tokens": 256,
        "temperature": 0.5,
        "top_p": 1.0,
        "n": 1,
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

    let response_text = response.text().await?;
    let json_data: serde_json::Value = serde_json::from_str(&response_text)?;
    let summary = json_data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .to_string();

    Ok(summary)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MessageInfo {
    message: String,
    speaker: String,
    time: f64,
    timestring: String,
    uuid: String,
    vector: Vec<f32>,
}

#[derive(Debug, Clone)]
struct Memory {
    message_info: MessageInfo,
    similarity: f32,
}

fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let a_magnitude: f32 = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let b_magnitude: f32 = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    dot_product / (a_magnitude * b_magnitude)
}

async fn fetch_memories(
    input_vector: &Vec<f32>,
    input_uuid: &str,
    top_k: usize,
) -> Result<Vec<Memory>, Box<dyn std::error::Error>> {
    let mut memories: Vec<Memory> = Vec::new();
    let mut similarities: Vec<f32> = Vec::new();

    let mut entries = fs::read_dir("log").await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_path = entry.path();
        if let Some(ext) = file_path.extension() {
            if ext == "json" {
                let file_contents = fs::read_to_string(file_path).await?;
                let message_info: MessageInfo = serde_json::from_str(&file_contents)?;

                if message_info.uuid != input_uuid {
                    let similarity = cosine_similarity(&input_vector, &message_info.vector);
                    memories.push(Memory {
                        message_info,
                        similarity,
                    });
                    similarities.push(similarity);
                }
            }
        }
    }

    // Sort and select the top_k memories
    let mut memory_indices: Vec<usize> = (0..memories.len()).collect();
    memory_indices
        .sort_unstable_by(|a, b| similarities[*b].partial_cmp(&similarities[*a]).unwrap());
    let top_k_memories = memory_indices
        .into_iter()
        .take(top_k)
        .map(|i| memories[i].clone())
        .collect();

    Ok(top_k_memories)
}

async fn get_vector_from_json(
    json_data: serde_json::Value,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let embedding: Vec<f32> = json_data["data"][0]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    Ok(embedding)
}

async fn save_embeddings_to_json(
    json_data: serde_json::Value,
    input_message: &str,
    speaker: &str,
) -> Result<(Vec<f32>, String), Box<dyn std::error::Error>> {
    let timestamp = serde_json::to_value(chrono::Utc::now().timestamp_millis())?;
    let timestring = chrono::Utc::now().to_rfc2822();
    let uuid = Uuid::new_v4().to_string();

    let embedding: Vec<f32> = json_data["data"][0]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();

    let message = format!(
        "{}: {} - {}",
        speaker.to_string(),
        timestring,
        input_message.to_string()
    );
    //println!("{:?}", embedding);
    let message_info = MessageInfo {
        message,
        speaker: speaker.to_string(),
        time: timestamp.as_f64().unwrap(),
        timestring,
        uuid: uuid.clone(),
        vector: embedding.clone(),
    };
    let json_string = serde_json::to_string_pretty(&message_info)?;
    //println!("{:?}", json_string);

    let file_path = format!(
        "log/log_{}_{}.json",
        message_info.time, message_info.speaker
    );
    //println!("{}", file_path);
    let mut file = fs::File::create(file_path).await?;
    file.write_all(json_string.as_bytes()).await?;
    Ok((embedding, uuid))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").unwrap();
    let api_url = "https://api.openai.com/v1/chat/completions";
    let api_url_embedding = "https://api.openai.com/v1/embeddings";

    loop {
        // 用户输入content内容
        print!("🧑‍💻：");
        io::stdout().flush().unwrap();
        let mut content = String::new();
        stdin()
            .read_line(&mut content)
            .expect("Error reading input");
        let content = content.trim().to_string(); // 移除行尾的换行符
        let mut result_text = content.clone();
        print!("🦾🤖：");

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
        //println!("{}", response_text);
        let json_data: serde_json::Value = serde_json::from_str(&response_text)?;
        let (test_vec, input_uuid) =
            save_embeddings_to_json(json_data.clone(), &result_text, "USER").await?;

        //let (test_vec, input_uuid) = get_vector_from_json(json_data).await?;
        //println!("{:?}", test_vec);
        // Get the most similar memories
        let top_k_memories = 9;
        let similar_memories = fetch_memories(&test_vec, &input_uuid, top_k_memories).await?;
        // Summarize the memories
        let memory_summary = summarize_memories(&similar_memories, &api_key).await?;
        let memory_summary = format!(
            "这是相关历史对话的总结: {} - USER: {}",
            memory_summary, content
        );
        println!("memory_summary: {}", memory_summary);

        // Add the memory summary to the conversation
        let messages = vec![
            serde_json::json!({ "role": "system", "content": "我是一个名为GPT3的AI。我的主要目标是帮助用户规划、头脑风暴、概述以及构建他们的小说作品。"}),
            serde_json::json!({ "role": "user", "content": memory_summary }),
        ];

        let payload = serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": messages,
            "temperature": 1.0,
            "top_p": 1.0,
            "n": 1,
            "stream": true,
            "presence_penalty": 0,
            "frequency_penalty": 0
        });

        //println!("similar_memories:{:#?}", similar_memories);

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
        result_text = String::new();
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
                Err(_e) => {}
            }
        }
        print!("\n");
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
        //println!("{}", response_text);
        let json_data: serde_json::Value = serde_json::from_str(&response_text)?;
        //println!("{}", result_text);
        save_embeddings_to_json(json_data, &result_text, "GPT3").await?;
    }
}
