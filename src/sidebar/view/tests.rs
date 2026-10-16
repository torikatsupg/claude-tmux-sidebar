use super::*;
use crate::i18n::{EN, JA};
use crate::listing::Group;
use crate::session::Record;
use crate::session::live::Claude;
use crate::sidebar::rows::RowKey;
use crate::sidebar::selection::Selection;
use crate::sidebar::snapshot::Snapshot;
use crate::text;
use crate::tmux::pane::Pane;
use crate::tmux::pane::fixture::pane;

const WIDTH: u16 = 34;

fn record(state: Status, detail: &str) -> Record {
    Record {
        session_id: "a".to_owned(),
        state,
        detail: detail.to_owned(),
        cwd: "/src/api".to_owned(),
        since: Some(100),
        ..Record::default()
    }
}

fn model(state: Status, detail: &str, selected: bool) -> Model {
    let claude = Claude {
        record: record(state, detail),
        pane: Some(pane(1, 2)),
    };
    model_of(claude, selected)
}

fn model_of(claude: Claude, selected: bool) -> Model {
    let groups = [Group {
        name: GroupName::Session("work".to_owned()),
        claudes: vec![claude],
    }];
    let snapshot = Snapshot {
        rows: rows::rows(&groups),
        ..Snapshot::default()
    };
    let selection = Selection {
        selected: selected.then(|| RowKey::Claude("a".to_owned())),
        synced: None,
    };
    Model {
        snapshot,
        selection,
        now: 102,
        height: 8,
        help: true,
        ..Model::default()
    }
}

fn text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

fn screen(model: &Model, messages: &Messages) -> Vec<String> {
    lines(model, messages, WIDTH, model.height)
        .iter()
        .map(text)
        .collect()
}

#[test]
fn draws_title_list_and_help() {
    let screen = screen(&model(Status::Waiting, "Edit", false), &EN);
    assert_eq!(screen.len(), 8);
    assert_eq!(screen.first().map(String::as_str), Some(" Claude ◆1 "));
    assert_eq!(screen.get(2).map(String::as_str), Some(" ▾ work"));
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ◆ api                    Edit 2s ")
    );
    assert_eq!(
        screen.last().map(String::as_str),
        Some(" ⏎ jump  j/k move  q close")
    );
}

#[test]
fn hiding_the_help_gives_its_line_to_the_list() {
    let model = Model {
        help: false,
        ..model(Status::Waiting, "Edit", false)
    };
    let screen = screen(&model, &EN);
    assert_eq!(screen.len(), 8);
    assert!(screen.iter().all(|line| !line.contains("jump")));
    assert_eq!(model.list_end(model.height), 8);
}

#[test]
fn the_session_in_this_window_is_marked() {
    let mut model = model(Status::Done, "", false);
    model.snapshot.window = Some(pane(1, 2).window_id);
    assert_eq!(
        screen(&model, &EN).get(3).map(String::as_str),
        Some("▌✓ api                         2s ")
    );
}

#[test]
fn rows_fill_the_width_and_hide_idle_details() {
    let row = screen(&model(Status::Idle, "Bash", false), &JA)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(text::width(&row), usize::from(WIDTH));
    assert!(row.ends_with(" 2s "), "{row}");
    assert!(!row.contains("Bash"), "{row}");
}

#[test]
fn the_cursor_is_a_background_that_keeps_the_row_colors() {
    let model = model(Status::Done, "", true);
    let line = lines(&model, &EN, WIDTH, model.height)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(line.width(), usize::from(WIDTH));
    assert!(line.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    assert!(line.spans.iter().any(|s| s.style.fg == Some(Color::Cyan)));
}

fn busy_claude(windows: u32, context: Option<u8>) -> Claude {
    Claude {
        record: Record {
            branch: "feat/rows".to_owned(),
            prompt: "fix the failing sidebar test please".to_owned(),
            context,
            ..record(Status::Done, "")
        },
        pane: Some(Pane {
            session_windows: windows,
            ..pane(1, 2)
        }),
    }
}

#[test]
fn a_session_shows_its_branch_then_its_prompt_and_context() {
    let screen = screen(&model_of(busy_claude(1, Some(42)), false), &EN);
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ✓ api feat/rows               2s ")
    );
    assert_eq!(
        screen.get(4).map(String::as_str),
        Some(" └ fix the failing sidebar t… 42% ")
    );
}

#[test]
fn the_window_index_shows_only_with_several_windows() {
    let screen = screen(&model_of(busy_claude(3, None), false), &EN);
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ✓ 2:api feat/rows             2s ")
    );
    assert_eq!(
        screen.get(4).map(String::as_str),
        Some(" └ fix the failing sidebar test … ")
    );
}

#[test]
fn a_narrow_row_cuts_the_branch_then_drops_it() {
    let model = model_of(busy_claude(1, Some(90)), true);
    let row = |width| lines(&model, &EN, width, model.height).get(3).map(text);
    assert_eq!(row(16).as_deref(), Some(" ✓ api feat… 2s "));
    assert_eq!(row(14).as_deref(), Some(" ✓ api     2s "));
    let lines = lines(&model, &EN, 16, model.height);
    let detail = lines.get(4).cloned().unwrap_or_default();
    assert!(detail.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    assert!(detail.spans.iter().any(|s| s.style.fg == Some(Color::Red)));
}
