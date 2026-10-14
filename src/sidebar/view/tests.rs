use super::*;
use crate::i18n::{EN, JA};
use crate::listing::Group;
use crate::session::Record;
use crate::session::live::Claude;
use crate::sidebar::rows::RowKey;
use crate::sidebar::selection::Selection;
use crate::sidebar::snapshot::Snapshot;
use crate::text;
use crate::tmux::pane::fixture::pane;

const WIDTH: u16 = 34;

fn model(state: Status, detail: &str, selected: bool) -> Model {
    let record = Record {
        session_id: "a".to_owned(),
        state,
        detail: detail.to_owned(),
        cwd: "/src/api".to_owned(),
        since: Some(100),
        ..Record::default()
    };
    let groups = [Group {
        name: GroupName::Session("work".to_owned()),
        claudes: vec![Claude {
            record,
            pane: Some(pane(1, 2)),
        }],
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
        Some(" ◆ 2:api          Edit waiting 2s ")
    );
    assert_eq!(
        screen.last().map(String::as_str),
        Some(" ⏎ jump  j/k move  q close")
    );
}

#[test]
fn rows_fill_the_width_and_hide_idle_details() {
    let row = screen(&model(Status::Idle, "Bash", false), &JA)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(text::width(&row), usize::from(WIDTH));
    assert!(row.ends_with(" 待機 2s "), "{row}");
}

#[test]
fn the_selected_row_is_reversed_across_the_width() {
    let model = model(Status::Done, "", true);
    let line = lines(&model, &EN, WIDTH, model.height)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(line.width(), usize::from(WIDTH));
    assert!(line.style.add_modifier.contains(Modifier::REVERSED));
}
