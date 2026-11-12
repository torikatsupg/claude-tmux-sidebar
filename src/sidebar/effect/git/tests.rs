use super::*;
use crate::sidebar::preview::Changes;

#[test]
fn git_runs_without_locks_fsmonitor_or_diff_drivers() {
    assert_eq!(
        args("/src", &["diff", "--shortstat", "HEAD"]),
        [
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-C",
            "/src",
            "diff",
            "--shortstat",
            "HEAD",
            "--no-textconv",
            "--no-ext-diff"
        ]
    );
    assert_eq!(
        args("/src", &["rev-parse"]).last().map(String::as_str),
        Some("rev-parse")
    );
}

#[test]
fn a_slow_command_is_given_up() {
    let started = Instant::now();
    assert_eq!(
        run("sleep", &["5".to_owned()], Duration::from_millis(100)),
        None
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        run("echo", &["hi".to_owned()], Duration::from_secs(2)).as_deref(),
        Some("hi")
    );
}

#[test]
fn the_answer_holds_the_branch_and_changes_or_nothing() {
    let stat = " 2 files changed, 5 insertions(+), 1 deletion(-)";
    assert_eq!(
        answer("/src", Some("main".to_owned()), Some(stat)),
        Msg::Git(GitRead {
            dir: "/src".to_owned(),
            branch: "main".to_owned(),
            changes: Some(Changes {
                files: 2,
                added: 5,
                removed: 1
            })
        })
    );
    assert_eq!(
        answer("/tmp", None, None),
        Msg::Git(GitRead {
            dir: "/tmp".to_owned(),
            branch: String::new(),
            changes: None
        })
    );
}
