//! What Claude said last: the text of its last reply, and the tool call it waits on.

use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;

/// Claude's last words in a transcript, for the sidebar's preview.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Said {
    /// The text blocks of the last main-thread assistant message with text, without thinking or tool calls.
    pub reply: String,
    /// The main-thread tool calls without a result since the user's last prompt, in call order: among them
    /// is what a waiting session asks for, or the tool a running one is using.
    pub pending: Vec<Ask>,
}

/// A tool call and its main input: the command, the file, the question, or one telling field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ask {
    pub tool: String,
    pub detail: String,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default, rename = "isSidechain")]
    sidechain: bool,
    message: Option<Message>,
}

#[derive(Deserialize)]
struct Message {
    #[serde(default)]
    content: Value,
}

/// The fields that name what a tool call is about, by tool; any other tool shows the first of [`FIELDS`].
const FIELDS: [&str; 6] = ["url", "pattern", "path", "query", "description", "prompt"];

/// Reads `text` (JSON lines) for what Claude said last.
pub fn said(text: &str) -> Said {
    let entries: Vec<(String, Vec<Value>)> = text
        .lines()
        .filter(|line| line.contains("\"assistant\"") || line.contains("\"user\""))
        .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
        .filter(|e| !e.sidechain)
        .filter_map(|e| Some((e.kind, blocks(e.message?.content))))
        .collect();
    Said {
        reply: reply(&entries),
        pending: pending(since_prompt(&entries)),
    }
}

/// A message's content blocks; a prompt typed as plain text is one text block.
fn blocks(content: Value) -> Vec<Value> {
    match content {
        Value::Array(blocks) => blocks,
        Value::String(text) => vec![serde_json::json!({"type": "text", "text": text})],
        _ => Vec::new(),
    }
}

/// The entries after the user's last prompt (a user message with text rather than tool results).
fn since_prompt(entries: &[(String, Vec<Value>)]) -> &[(String, Vec<Value>)] {
    let prompt = entries.iter().rposition(|(kind, blocks)| {
        kind == "user"
            && of_kind(blocks, "text").next().is_some()
            && of_kind(blocks, "tool_result").next().is_none()
    });
    prompt.map_or(entries, |i| entries.get(i + 1..).unwrap_or_default())
}

fn of_kind<'a>(blocks: &'a [Value], kind: &'a str) -> impl Iterator<Item = &'a Value> {
    blocks
        .iter()
        .filter(move |b| b.get("type").and_then(Value::as_str) == Some(kind))
}

fn reply(entries: &[(String, Vec<Value>)]) -> String {
    entries
        .iter()
        .rev()
        .filter(|(kind, _)| kind == "assistant")
        .map(|(_, blocks)| {
            of_kind(blocks, "text")
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .find(|text| !text.is_empty())
        .unwrap_or_default()
}

fn pending(entries: &[(String, Vec<Value>)]) -> Vec<Ask> {
    let answered: HashSet<&str> = entries
        .iter()
        .filter(|(kind, _)| kind == "user")
        .flat_map(|(_, blocks)| of_kind(blocks, "tool_result"))
        .filter_map(|b| b.get("tool_use_id").and_then(Value::as_str))
        .collect();
    entries
        .iter()
        .filter(|(kind, _)| kind == "assistant")
        .flat_map(|(_, blocks)| of_kind(blocks, "tool_use"))
        .filter(|b| {
            let id = b.get("id").and_then(Value::as_str).unwrap_or_default();
            !answered.contains(id)
        })
        .map(|b| {
            let tool = b.get("name").and_then(Value::as_str).unwrap_or_default();
            let input = b.get("input").unwrap_or(&Value::Null);
            Ask {
                tool: tool.to_owned(),
                detail: detail(tool, input),
            }
        })
        .collect()
}

fn detail(tool: &str, input: &Value) -> String {
    let field = |name: &str| input.get(name).and_then(Value::as_str);
    let found = match tool {
        "Bash" => field("command"),
        "Edit" | "Write" | "MultiEdit" | "Read" => field("file_path"),
        "NotebookEdit" => field("notebook_path"),
        "AskUserQuestion" => input
            .pointer("/questions/0/question")
            .and_then(Value::as_str),
        _ => FIELDS.iter().find_map(|name| field(name)),
    };
    found.unwrap_or_default().to_owned()
}

#[cfg(test)]
mod tests;
