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
    #[serde(default)]
    pub cwd: String,
}

/// A hook event, with the data each one carries.
#[derive(Debug, PartialEq, Eq)]
pub enum Event<'a> {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse { tool: &'a str },
    PostToolUse { tool: &'a str },
    PermissionRequest { tool: &'a str },
    Notification { kind: &'a str },
    Stop,
    Other,
}

impl<'a> Event<'a> {
    pub fn of(payload: &'a Payload) -> Self {
        let tool = payload.tool_name.as_str();
        match payload.hook_event_name.as_str() {
            "SessionStart" => Self::SessionStart,
            "SessionEnd" => Self::SessionEnd,
            "UserPromptSubmit" => Self::UserPromptSubmit,
            "PreToolUse" => Self::PreToolUse { tool },
            "PostToolUse" => Self::PostToolUse { tool },
            "PermissionRequest" => Self::PermissionRequest { tool },
            "Notification" => Self::Notification {
                kind: &payload.notification_type,
            },
            "Stop" => Self::Stop,
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
        assert_eq!(Event::of(&payload), Event::PreToolUse { tool: "Bash" });
    }
}
