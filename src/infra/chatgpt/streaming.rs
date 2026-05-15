//! Streaming SSE response handling for ChatGPT helper requests.

use super::{AccessContext, CHATGPT_RESPONSES_URL, HelperRequest, chatgpt_headers};
use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use serde_json::Value;

/// Streams a helper response from ChatGPT while reporting partial text.
pub async fn stream_helper_response<F>(
    access: &AccessContext,
    request: HelperRequest,
    mut on_update: F,
) -> Result<String>
where
    F: FnMut(String) + Send,
{
    let mut content = vec![serde_json::json!({"type":"input_text","text":request.prompt})];
    if let Some(image) = request
        .image_data_url
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        content.push(serde_json::json!({"type":"input_image","image_url":image}));
    }
    let client = reqwest::Client::new();
    let response = client
        .post(CHATGPT_RESPONSES_URL)
        .headers(chatgpt_headers(access, "text/event-stream", true)?)
        .json(&serde_json::json!({
            "model": request.model,
            "input": [{"type":"message","role":"user","content":content}],
            "stream": true, "store": false,
            "include": ["reasoning.encrypted_content"],
            "text": {"verbosity": request.response_style},
            "reasoning": {"effort": request.thinking_variant, "summary": "auto"},
            "instructions": if request.instructions.trim().is_empty() { "." } else { request.instructions.trim() }
        }))
        .send().await.context("Could not reach ChatGPT")?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "ChatGPT request failed with status {status}. {}",
            body.chars().take(240).collect::<String>()
        ));
    }
    let mut text = String::new();
    let mut completed_text = String::new();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Could not read ChatGPT response stream")?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        let lines: Vec<_> = buffer
            .split('\n')
            .map(|l| l.trim_end_matches('\r').to_owned())
            .collect();
        let n = lines.len().saturating_sub(1);
        for line in lines.iter().take(n) {
            if let Some(p) = parse_sse_line(line) {
                if !p.delta.is_empty() {
                    text.push_str(&p.delta);
                    on_update(text.trim().to_owned());
                }
                if !p.completed_text.is_empty() {
                    completed_text = p.completed_text;
                }
            }
        }
        buffer = lines.last().cloned().unwrap_or_default();
    }
    if let Some(p) = parse_sse_line(&buffer) {
        text.push_str(&p.delta);
        if !p.completed_text.is_empty() {
            completed_text = p.completed_text;
        }
    }
    let final_text = if text.trim().is_empty() {
        completed_text.trim().to_owned()
    } else {
        text.trim().to_owned()
    };
    Ok(if final_text.is_empty() {
        "No response text was returned.".to_owned()
    } else {
        final_text
    })
}

struct SsePart {
    delta: String,
    completed_text: String,
}

/// Parses one server-sent event line from the ChatGPT stream.
fn parse_sse_line(line: &str) -> Option<SsePart> {
    if !line.starts_with("data:") {
        return None;
    }
    let payload = line.trim_start_matches("data:").trim();
    if payload.is_empty() || payload == "[DONE]" {
        return None;
    }
    let event: Value = serde_json::from_str(payload).ok()?;
    let delta = extract_delta_text(&event).unwrap_or_default();
    let completed_text = if event.get("type").and_then(Value::as_str) == Some("response.completed")
    {
        extract_completed_text(event.get("response").unwrap_or(&event))
    } else {
        String::new()
    };
    Some(SsePart {
        delta,
        completed_text,
    })
}

/// Extracts incremental output text from a stream event.
fn extract_delta_text(event: &Value) -> Option<String> {
    let t = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !(t == "response.output_text.delta"
        || (t.ends_with(".delta")
            && t.contains("output")
            && t.contains("text")
            && !t.contains("reasoning")))
    {
        return None;
    }
    event
        .get("delta")
        .and_then(Value::as_str)
        .or_else(|| {
            event
                .get("delta")
                .and_then(|v| v.get("text"))
                .and_then(Value::as_str)
        })
        .or_else(|| event.get("text").and_then(Value::as_str))
        .map(str::to_owned)
}

/// Extracts final output text from a completed response payload.
fn extract_completed_text(root: &Value) -> String {
    root.get("output")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|i| i.get("type").and_then(Value::as_str) == Some("message"))
                .filter_map(|m| m.get("content").and_then(Value::as_array))
                .flat_map(|c| c.iter())
                .filter(|p| p.get("type").and_then(Value::as_str) == Some("output_text"))
                .filter_map(|p| p.get("text").and_then(Value::as_str))
                .collect::<String>()
        })
        .unwrap_or_default()
}
