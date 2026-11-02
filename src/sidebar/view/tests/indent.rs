use super::*;
use crate::sidebar::headings::with_headings;
use crate::sidebar::indent::Indent;
use crate::sidebar::label::{Label, SingleWindow};

fn tree(spaces: &str) -> Model {
    let mut claude = busy_claude(2, None);
    claude.pane = claude.pane.map(|p| Pane {
        window_named: true,
        window_name: "mywork".to_owned(),
        ..p
    });
    let model = model_of(claude, false);
    let label = Label::Auto(SingleWindow::Inline);
    let rows = with_headings(&model.snapshot.rows, label);
    Model {
        label,
        indent: Indent::parse(Some(spaces)),
        snapshot: Snapshot {
            rows,
            ..model.snapshot
        },
        ..model
    }
}

#[test]
fn the_indent_option_sets_the_spaces_per_level_of_the_tree() {
    for (value, unit) in [("0", ""), ("1", " "), ("2", "  ")] {
        let screen = screen(&tree(value), &EN);
        let row = |i: usize| screen.get(i).cloned().unwrap_or_default();
        assert_eq!(row(3), format!("{unit}2:mywork"));
        assert!(row(4).starts_with(&format!("{unit}{unit}✓ ")), "{}", row(4));
        assert!(row(5).starts_with(&format!("{unit}{unit}└ ")), "{}", row(5));
    }
}

#[test]
fn indented_rows_keep_the_width_at_every_indent() {
    for value in ["0", "2", "4"] {
        let screen = screen(&tree(value), &EN);
        let widths: Vec<_> = screen
            .iter()
            .skip(4)
            .take(2)
            .map(|r| text::width(r))
            .collect();
        assert_eq!(widths, [usize::from(WIDTH); 2], "{value}");
    }
}

#[test]
fn a_window_without_claude_is_indented_one_level() {
    let window = Pane {
        window_name: "zsh".to_owned(),
        ..pane(4, 3)
    };
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    for (value, indent) in [("0", ""), ("2", "  ")] {
        let model = Model {
            label: Label::Auto(SingleWindow::Inline),
            indent: Indent::parse(Some(value)),
            snapshot: Snapshot {
                rows: rows::with_windows(&groups, std::slice::from_ref(&window)),
                ..Snapshot::default()
            },
            height: 8,
            ..Model::default()
        };
        let row = screen(&model, &EN).get(3).cloned();
        assert_eq!(row, Some(format!("{indent}3:zsh")));
    }
}
