use super::fixture::pane;
use super::*;

#[test]
fn ids_round_trip() {
    assert_eq!(
        PaneId::parse("%12").map(|p| p.to_string()),
        Some("%12".to_owned())
    );
    assert_eq!(
        WindowId::parse("@3").map(|w| w.to_string()),
        Some("@3".to_owned())
    );
    assert_eq!(PaneId::parse("12"), None);
}

#[test]
fn parses_list_panes_output() {
    let panes = parse(
        "%1\twork\t2\t0\t@4\t\t1\t0\t1\t3\t1\tnvim\t/src\t1\tvim\n%2\twork\t2\t1\t@4\t1\t0\t1\t1\t3\t0\tzsh\t/\t0\ta\tb\nbroken",
    );
    assert_eq!(panes.len(), 2);
    assert!(panes.first().is_some_and(|p| p.active
        && !p.sidebar
        && p.window_index == 2
        && p.window_name == "vim"
        && p.session_windows == 3
        && p.attached
        && p.command == "nvim"
        && p.path == "/src"));
    assert!(panes.get(1).is_some_and(|p| p.sidebar
        && p.last
        && p.window_named
        && !p.attached
        && p.window_name == "a\tb"));
}

#[test]
fn focused_prefers_active_then_last() {
    let sidebar = Pane {
        sidebar: true,
        active: true,
        ..pane(1, 0)
    };
    let plain = [sidebar.clone(), pane(2, 0), pane(3, 0)];
    let with_last = [
        sidebar,
        pane(2, 0),
        Pane {
            last: true,
            ..pane(3, 0)
        },
    ];
    assert_eq!(focused(&plain, WindowId(0)).map(|p| p.id), Some(PaneId(2)));
    assert_eq!(
        focused(&with_last, WindowId(0)).map(|p| p.id),
        Some(PaneId(3))
    );
}
