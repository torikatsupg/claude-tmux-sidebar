use super::*;

fn assistant(content: &str, sidechain: bool) -> String {
    format!(
        r#"{{"type":"assistant","isSidechain":{sidechain},"message":{{"role":"assistant","content":[{content}]}}}}"#
    )
}

fn text(text: &str) -> String {
    format!(r#"{{"type":"text","text":"{text}"}}"#)
}

fn tool_use(id: &str, name: &str, input: &str) -> String {
    format!(r#"{{"type":"tool_use","id":"{id}","name":"{name}","input":{input}}}"#)
}

fn result(id: &str) -> String {
    format!(
        r#"{{"type":"user","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"{id}","content":"ok"}}]}}}}"#
    )
}

fn ask(tool: &str, detail: &str) -> Ask {
    Ask {
        tool: tool.to_owned(),
        detail: detail.to_owned(),
    }
}

#[test]
fn the_reply_is_the_text_of_the_last_main_thread_answer() {
    let lines = [
        assistant(&text("old answer"), false),
        assistant(
            r#"{"type":"thinking","thinking":"hmm","signature":"x"}"#,
            false,
        ),
        assistant(
            &format!("{},{}", text("Done.\\nAll tests pass."), text("Bye")),
            false,
        ),
        assistant(&tool_use("t1", "Bash", r#"{"command":"ls"}"#), false),
        result("t1"),
        assistant(&text("from a subagent"), true),
        r#"{"type":"user","message":{"role":"user","content":"next prompt"}}"#.to_owned(),
        "partial line".to_owned(),
    ];
    let said = said(&lines.join("\n"));
    assert_eq!(said.reply, "Done.\nAll tests pass.\n\nBye");
    assert!(said.pending.is_empty());
    assert_eq!(super::said("nothing"), Said::default());
}

#[test]
fn a_tool_use_without_a_result_is_what_claude_asks_for() {
    let lines = [
        assistant(&tool_use("t1", "Read", r#"{"file_path":"/a"}"#), false),
        result("t1"),
        assistant(
            &tool_use(
                "t2",
                "Bash",
                r#"{"command":"rm -rf build","description":"clean"}"#,
            ),
            false,
        ),
    ];
    assert_eq!(
        said(&lines.join("\n")).pending,
        [ask("Bash", "rm -rf build")]
    );
    assert!(said(&lines[..2].join("\n")).pending.is_empty());
}

#[test]
fn edits_and_writes_show_their_file() {
    let edit = assistant(
        &tool_use("e", "Edit", r#"{"file_path":"/src/a.rs","old_string":"x"}"#),
        false,
    );
    let write = assistant(
        &tool_use("w", "Write", r#"{"file_path":"/src/b.rs","content":"y"}"#),
        false,
    );
    assert_eq!(said(&edit).pending, [ask("Edit", "/src/a.rs")]);
    assert_eq!(said(&write).pending, [ask("Write", "/src/b.rs")]);
}

#[test]
fn other_tools_show_one_telling_field_or_just_their_name() {
    let fetch = assistant(
        &tool_use(
            "f",
            "WebFetch",
            r#"{"url":"https://example.com","prompt":"p"}"#,
        ),
        false,
    );
    let odd = assistant(&tool_use("o", "mcp__x__do", r#"{"count":3}"#), false);
    assert_eq!(
        said(&fetch).pending,
        [ask("WebFetch", "https://example.com")]
    );
    assert_eq!(said(&odd).pending, [ask("mcp__x__do", "")]);
}

#[test]
fn a_question_shows_its_text() {
    let input = r#"{"questions":[{"question":"Which database?","header":"DB","options":[]}]}"#;
    let question = assistant(&tool_use("q", "AskUserQuestion", input), false);
    assert_eq!(
        said(&question).pending,
        [ask("AskUserQuestion", "Which database?")]
    );
}

#[test]
fn a_subagent_tool_use_is_not_the_main_ask() {
    let side = assistant(&tool_use("s", "Bash", r#"{"command":"ls"}"#), true);
    assert!(said(&side).pending.is_empty());
}

fn prompt(text: &str) -> String {
    format!(r#"{{"type":"user","message":{{"role":"user","content":"{text}"}}}}"#)
}

#[test]
fn only_calls_since_the_last_prompt_are_pending() {
    let lines = [
        assistant(&tool_use("t1", "Bash", r#"{"command":"old"}"#), false),
        prompt("next task"),
        assistant(&tool_use("t2", "Bash", r#"{"command":"new"}"#), false),
    ];
    assert_eq!(said(&lines.join("\n")).pending, [ask("Bash", "new")]);
    assert!(said(&lines[..2].join("\n")).pending.is_empty());
}

#[test]
fn parallel_calls_stay_pending_in_call_order() {
    let calls = format!(
        "{},{}",
        tool_use("r", "Read", r#"{"file_path":"/a"}"#),
        tool_use("b", "Bash", r#"{"command":"make"}"#)
    );
    let lines = [assistant(&calls, false), result("r")];
    assert_eq!(
        said(&assistant(&calls, false)).pending,
        [ask("Read", "/a"), ask("Bash", "make")]
    );
    assert_eq!(said(&lines.join("\n")).pending, [ask("Bash", "make")]);
}
