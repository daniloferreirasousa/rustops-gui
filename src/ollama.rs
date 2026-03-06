use reqwest::blocking::Client;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;

use crate::storage::ChatMessage; // <--- Importa o modelo de dados 

// Envia todo o histórico para a API de chat do Ollama
pub fn send_to_ollama_chat(history: Vec<ChatMessage>, tx: Sender<String>) {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .unwrap_or_else(|_| Client::new());

    // Corpo da requisição
    let request_body = serde_json::json!({
        "model": "rustops",
        "messages": history,
        "stream": true
    });

    let response = match client
        .post("http://127.0.0.1:11434/api/chat")
        .json(&request_body)
        .send()
    {
        Ok(res) => res,
        Err(e) => {
            let _ = tx.send(format!(" [Erro: {}]", e));
            let _ = tx.send("[FIM]".to_string());
            return;
        }
    };

    let reader = BufReader::new(response);

    // Lê a resposta que vem fracionada em várias linhas de JSON
    for line in reader.lines() {
        if let Ok(line_str) = line {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line_str) {
                if let Some(texto_parcial) = parsed["message"]["content"].as_str() {
                    let _ = tx.send(texto_parcial.to_string());
                }

                if let Some(done) = parsed["done"].as_bool() {
                    if done { break; }
                }
            }
        }
    }

    // Sinaliza para o app.rs que terminour de falar
    let _ = tx.send("[FIM]".to_string());
}