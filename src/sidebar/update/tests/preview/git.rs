//! When the preview reads a directory with git, and what it keeps.

use super::*;

#[test]
fn git_runs_again_only_after_the_interval() {
    let first = loaded(
        Model {
            now: 100,
            ..model(Mode::Pane)
        },
        in_dirs(true),
    );
    let (soon, early) = update(Model { now: 105, ..first }, Msg::Loaded(in_dirs(true)));
    assert!(early.is_empty());
    let (_, later) = update(Model { now: 110, ..soon }, Msg::Loaded(in_dirs(true)));
    assert_eq!(later, [asking("/a", &["/b"])]);
}

#[test]
fn a_git_answer_is_kept_for_the_preview() {
    let model = loaded(model(Mode::Pane), windows(true));
    let changes = Some(Changes {
        files: 1,
        added: 2,
        removed: 0,
    });
    let read = GitRead {
        dir: "/w1".to_owned(),
        branch: "main".to_owned(),
        changes,
        tops: vec![("/w1".to_owned(), "/w".to_owned())],
    };
    let (model, effects) = update(model, Msg::Git(read));
    assert_eq!(model.git.as_ref().map(|g| g.branch.as_str()), Some("main"));
    assert_eq!(model.tops.get("/w1").map(String::as_str), Some("/w"));
    assert_eq!(model.git.and_then(|g| g.changes), changes);
    assert!(effects.is_empty());
}

#[test]
fn the_repositories_of_other_sessions_are_asked_for_once() {
    let first = loaded(
        Model {
            now: 100,
            ..model(Mode::Pane)
        },
        in_dirs(true),
    );
    let tops = vec![
        ("/a".to_owned(), "/a".to_owned()),
        ("/b".to_owned(), String::new()),
    ];
    let read = GitRead {
        dir: "/a".to_owned(),
        branch: String::new(),
        changes: None,
        tops,
    };
    let answered = update(first, Msg::Git(read)).0;
    let (_, later) = update(
        Model {
            now: 110,
            ..answered
        },
        Msg::Loaded(in_dirs(true)),
    );
    assert_eq!(later, [git("/a")]);
}
