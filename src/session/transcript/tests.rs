use super::*;

fn assistant(model: &str, branch: &str, tokens: u64, sidechain: bool) -> String {
    format!(
        r#"{{"type":"assistant","isSidechain":{sidechain},"gitBranch":"{branch}","requestedModel":"{model}","message":{{"model":"x","usage":{{"input_tokens":{tokens},"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}}}}"#
    )
}

#[test]
fn picks_the_last_main_thread_assistant_entry() {
    let text = [
        assistant("claude-opus-5-5", "old", 20_000, false),
        assistant("claude-opus-5-5", "main", 100_000, false),
        assistant("claude-haiku-5-5", "side", 10_000, true),
        r#"{"type":"user","message":{"content":"hi"}}"#.to_owned(),
        "partial line".to_owned(),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            branch: "main".to_owned(),
            usage: Some(used(100_000))
        })
    );
    assert_eq!(latest("nothing"), None);
}

fn used(tokens: u64) -> Usage {
    Usage {
        tokens,
        model: "claude-opus-5-5".to_owned(),
    }
}

fn boundary(branch: &str, sidechain: bool) -> String {
    format!(
        r#"{{"type":"system","subtype":"compact_boundary","isSidechain":{sidechain},"gitBranch":"{branch}","compactMetadata":{{}}}}"#
    )
}

#[test]
fn the_model_falls_back_to_the_message_model() {
    let text = assistant("", "main", 1_000, false);
    let usage = latest(&text).and_then(|l| l.usage).map(|u| u.model);
    assert_eq!(usage.as_deref(), Some("x"));
}

#[test]
fn a_compaction_after_the_last_assistant_entry_leaves_the_context_unknown() {
    let text = [
        assistant("claude-opus-5-5", "old", 180_000, false),
        boundary("after", false),
        r#"{"type":"user","message":{"content":"summary"}}"#.to_owned(),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            branch: "after".to_owned(),
            usage: None
        })
    );
}

#[test]
fn an_assistant_entry_after_a_compaction_wins() {
    let text = [
        boundary("old", false),
        assistant("claude-opus-5-5", "new", 20_000, false),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            branch: "new".to_owned(),
            usage: Some(used(20_000))
        })
    );
}

#[test]
fn only_a_main_thread_boundary_entry_counts_as_a_compaction() {
    let text = [
        assistant("claude-opus-5-5", "main", 100_000, false),
        boundary("side", true),
        r#"{"type":"user","message":{"content":"what is compact_boundary?"}}"#.to_owned(),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            branch: "main".to_owned(),
            usage: Some(used(100_000))
        })
    );
}
