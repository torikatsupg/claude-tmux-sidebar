//! UI strings, chosen by the `@cts_lang` tmux option.

/// Every user-facing string of the sidebar.
pub struct Messages {
    pub waiting: &'static str,
    pub running: &'static str,
    pub done: &'static str,
    pub idle: &'static str,
    pub outside: &'static str,
    pub no_sessions: &'static str,
    pub none: &'static str,
    pub help: &'static str,
    pub nothing: &'static str,
}

pub const EN: Messages = Messages {
    waiting: "waiting",
    running: "running",
    done: "done",
    idle: "idle",
    outside: "(outside tmux)",
    no_sessions: "No sessions",
    none: "none",
    help: "⏎ jump  j/k move  q close",
    nothing: "No done or waiting Claude sessions",
};

pub const JA: Messages = Messages {
    waiting: "承認待ち",
    running: "稼働中",
    done: "完了",
    idle: "待機",
    outside: "(tmux外)",
    no_sessions: "セッションなし",
    none: "なし",
    help: "⏎移動 j/k選択 q閉じる",
    nothing: "完了・承認待ちの Claude はありません",
};

/// Messages for a language code; English for anything unknown.
pub fn messages(lang: Option<&str>) -> &'static Messages {
    match lang {
        Some("ja") => &JA,
        _ => &EN,
    }
}
