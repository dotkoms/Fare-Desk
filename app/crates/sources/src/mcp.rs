//! A minimal client for remote MCP servers that answer `tools/call` over plain
//! HTTP, as Kiwi.com's and Skiplagged's public servers do. One stateless request
//! per call; the answer may be JSON or a server-sent event stream.

use serde_json::{json, Value};

use crate::SourceError;

/// Call `tool` with `arguments` and return its structured result.
pub async fn call_tool(
    client: &reqwest::Client,
    url: &str,
    source: &str,
    tool: &str,
    arguments: Value,
    timeout_secs: u64,
) -> Result<Value, SourceError> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": tool, "arguments": arguments },
    });
    let response = client
        .post(url)
        .header("accept", "application/json, text/event-stream")
        .json(&request)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|error| SourceError::Unavailable(format!("{source} request failed: {error}")))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| SourceError::Unavailable(format!("{source} response: {error}")))?;
    if !status.is_success() {
        let snippet: String = body.chars().take(200).collect();
        return Err(SourceError::Unavailable(format!(
            "{source} returned {status}: {snippet}"
        )));
    }
    structured_result(&body, source)
}

/// The tool's structured result from a JSON body or an event stream. A tool-level
/// error (`isError`) is the caller's request being refused, not an outage.
pub fn structured_result(body: &str, source: &str) -> Result<Value, SourceError> {
    let payload = body
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .find(|data| data.starts_with('{'))
        .unwrap_or(body.trim());
    let envelope: Value = serde_json::from_str(payload)
        .map_err(|error| SourceError::Parse(format!("{source} response is not JSON: {error}")))?;
    if let Some(error) = envelope.get("error") {
        return Err(SourceError::Unavailable(format!("{source} error: {error}")));
    }
    let result = envelope
        .get("result")
        .ok_or_else(|| SourceError::Parse(format!("{source} response has no result")))?;
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        let message = result
            .pointer("/content/0/text")
            .and_then(Value::as_str)
            .unwrap_or("search failed");
        return Err(SourceError::InvalidRequest(format!("{source}: {message}")));
    }
    match result.get("structuredContent") {
        Some(value) => Ok(value.clone()),
        None => {
            let text = result
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .ok_or_else(|| SourceError::Parse(format!("{source} result has no content")))?;
            serde_json::from_str(text)
                .map_err(|error| SourceError::Parse(format!("{source} content: {error}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_event_stream_and_a_plain_body() {
        let streamed = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"structuredContent\":{\"ok\":1}}}\n\n";
        assert_eq!(structured_result(streamed, "x").unwrap()["ok"], 1);
        let plain = r#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"{\"ok\":2}"}]}}"#;
        assert_eq!(structured_result(plain, "x").unwrap()["ok"], 2);
    }

    #[test]
    fn a_tool_error_is_a_refused_request() {
        let body = r#"data: {"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Unknown location"}],"isError":true}}"#;
        assert!(matches!(
            structured_result(body, "x"),
            Err(SourceError::InvalidRequest(_))
        ));
    }
}
