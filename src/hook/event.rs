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
    /// On `Stop`: the subagents, shells and monitors the session started in the background.
    #[serde(default)]
    pub background_tasks: Vec<BackgroundTask>,
}

/// One entry of `background_tasks`.
#[derive(Debug, Default, Deserialize)]
pub struct BackgroundTask {
    /// `subagent`, `shell`, `monitor`, …
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub status: String,
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
    /// `busy` is the kind of a background task still running, if any.
    /// Monitors do not count: they wait for events, and some (artifact subscriptions) never end.
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
            "Stop" => Self::Stop {
                busy: payload
                    .background_tasks
                    .iter()
                    .find(|t| t.status == "running" && t.kind != "monitor")
                    .map(|t| t.kind.as_str()),
            },
            "SubagentStop" => Self::SubagentStop { agent },
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_event_and_its_data() {
        let payload: Payload = serde_json::from_str(
            r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Bash","extra":1}"#,
        )
        .unwrap_or_default();
        assert_eq!(
            Event::of(&payload),
            Event::PreToolUse {
                tool: "Bash",
                agent: ""
            }
        );
    }

    #[test]
    fn reads_the_subagent_a_hook_fired_in() {
        let payload: Payload = serde_json::from_str(
            r#"{"hook_event_name":"PostToolUseFailure","session_id":"s","tool_name":"Bash",
                "agent_id":"a1","agent_type":"Explore"}"#,
        )
        .unwrap_or_default();
        assert_eq!(
            Event::of(&payload),
            Event::PostToolUse {
                tool: "Bash",
                agent: "a1"
            }
        );
    }

    #[test]
    fn stop_reads_the_running_background_task() {
        let payload: Payload = serde_json::from_str(
            r#"{"hook_event_name":"Stop","session_id":"s","background_tasks":[
                {"id":"a","type":"subagent","status":"completed"},
                {"id":"b","type":"shell","status":"running","command":"sleep 40"}]}"#,
        )
        .unwrap_or_default();
        assert_eq!(
            Event::of(&payload),
            Event::Stop {
                busy: Some("shell")
            }
        );
    }

    #[test]
    fn stop_does_not_count_a_running_monitor() {
        let payload: Payload = serde_json::from_str(
            r#"{"hook_event_name":"Stop","session_id":"s","background_tasks":[
                {"id":"a","type":"monitor","status":"running"}]}"#,
        )
        .unwrap_or_default();
        assert_eq!(Event::of(&payload), Event::Stop { busy: None });
    }
}
