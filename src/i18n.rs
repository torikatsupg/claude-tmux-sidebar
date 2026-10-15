//! UI strings, chosen by the `@cts_lang` tmux option.

/// Every user-facing string of the sidebar.
pub struct Messages {
    pub outside: &'static str,
    pub no_sessions: &'static str,
    pub none: &'static str,
    /// Key and what it does, for the help line.
    pub help: &'static [(&'static str, &'static str)],
    pub nothing: &'static str,
}

pub const EN: Messages = Messages {
    outside: "(outside tmux)",
    no_sessions: "No sessions",
    none: "none",
    help: &[("⏎", "jump"), ("j/k", "move"), ("q", "close")],
    nothing: "No done or waiting Claude sessions",
};

pub const JA: Messages = Messages {
    outside: "(tmux外)",
    no_sessions: "セッションなし",
    none: "なし",
    help: &[("⏎", "移動"), ("j/k", "選択"), ("q", "閉じる")],
    nothing: "完了・承認待ちの Claude はありません",
};

/// Messages for a language code; English for anything unknown.
pub fn messages(lang: Option<&str>) -> &'static Messages {
    match lang {
        Some("ja") => &JA,
        _ => &EN,
    }
}
