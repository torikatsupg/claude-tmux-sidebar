//! The Claude Code hook payload and the events the sidebar cares about.

use serde::Deserialize;

/// The fields of a hook payload that the sidebar reads; everything else is ignored.
#[derive(Debug, Default, Deserialize)]
pub struct Payload {
    #[serde(default)]
    pub hook_event_name: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub notification_type: String,
    #[serde(default)]
    pub tool_name: String,
    /// The subagent a hook fired in; absent on the main thread.
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub transcript_path: String,
    /// How `SessionStart` began: `startup`, `resume`, `clear` or `compact`.
    #[serde(default)]
    pub source: String,
    /// On `Stop` (and `SubagentStop`, for the whole session): the subagents, shells and monitors the session
    /// started in the background. `None` when the payload has no such field.
    #[serde(default)]
    pub background_tasks: Option<Vec<BackgroundTask>>,
    #[serde(default)]
    pub tool_use_id: String,
    #[serde(default)]
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub tool_response: serde_json::Value,
}

/// One entry of `background_tasks`.
#[derive(Debug, Default, Deserialize)]
pub struct BackgroundTask {
    #[serde(default)]
    pub id: String,
    /// `subagent`, `shell`, `monitor`, …
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    /// Only for `shell` tasks.
    #[serde(default)]
    pub command: String,
    /// Only for `subagent` tasks.
    #[serde(default)]
    pub agent_type: String,
}

/// A hook event, with the data each one carries. `agent` is the subagent it fired in, or empty for the main thread.
#[derive(Debug, PartialEq, Eq)]
pub enum Event<'a> {
    SessionStart {
        source: &'a str,
    },
    SessionEnd,
    UserPromptSubmit,
    PreToolUse {
        tool: &'a str,
        agent: &'a str,
    },
    /// Also `PostToolUseFailure`: either way the tool is no longer waiting.
    PostToolUse {
        tool: &'a str,
        agent: &'a str,
    },
    PermissionRequest {
        tool: &'a str,
        agent: &'a str,
    },
    Notification {
        kind: &'a str,
        agent: &'a str,
    },
    /// An MCP server asks the user for input during a tool call.
    Elicitation {
        agent: &'a str,
    },
    /// The user answered, declined or cancelled an MCP elicitation.
    ElicitationResult {
        agent: &'a str,
    },
    /// `busy` is the kind of a background task still running, if any.
    /// Monitors do not count: they wait for events, and some (artifact subscriptions) never end.
    /// Also `StopFailure` (the turn ended on an API error), which reports no background tasks.
    Stop {
        busy: Option<&'a str>,
    },
    SubagentStop {
        agent: &'a str,
    },
    Other,
}

impl<'a> Event<'a> {
    pub fn of(payload: &'a Payload) -> Self {
        let tool = payload.tool_name.as_str();
        let agent = payload.agent_id.as_str();
        match payload.hook_event_name.as_str() {
            "SessionStart" => Self::SessionStart {
                source: &payload.source,
            },
            "SessionEnd" => Self::SessionEnd,
            "UserPromptSubmit" => Self::UserPromptSubmit,
            "PreToolUse" => Self::PreToolUse { tool, agent },
            "PostToolUse" | "PostToolUseFailure" => Self::PostToolUse { tool, agent },
            "PermissionRequest" => Self::PermissionRequest { tool, agent },
            "Notification" => Self::Notification {
                kind: &payload.notification_type,
                agent,
            },
            "Elicitation" => Self::Elicitation { agent },
            "ElicitationResult" => Self::ElicitationResult { agent },
            "Stop" => Self::Stop {
                busy: payload
                    .background_tasks
                    .iter()
                    .flatten()
                    .find(|t| t.status == "running" && t.kind != "monitor")
                    .map(|t| t.kind.as_str()),
            },
            "StopFailure" => Self::Stop { busy: None },
            "SubagentStop" => Self::SubagentStop { agent },
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests;
