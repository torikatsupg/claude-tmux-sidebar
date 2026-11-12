use super::*;
use std::os::unix::fs::PermissionsExt;

#[test]
fn writes_private_files_and_reads_them_back() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state = dir.path().join("state");
    ensure_dir(&state)?;
    let record = Record {
        session_id: "s1".to_owned(),
        ..Record::default()
    };
    write(&state, &record)?;
    let mode = fs::metadata(path(&state, "s1"))?.permissions().mode() & 0o777;
    assert_eq!(mode, PRIVATE_FILE);
    assert_eq!(read(&state, "s1"), Some(record));
    assert_eq!(read_all(&state).len(), 1);
    Ok(())
}

#[test]
fn the_window_size_joins_its_record_and_leaves_with_it() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state = dir.path();
    let record = Record {
        session_id: "s1".to_owned(),
        ..Record::default()
    };
    write(state, &record)?;
    write_window(state, "s1", 1_000_000)?;
    assert_eq!(read_window(state, "s1"), Some(1_000_000));
    let mode = fs::metadata(state.join("s1.window"))?.permissions().mode() & 0o777;
    assert_eq!(mode, PRIVATE_FILE);
    let read = read_all(state);
    assert_eq!(read.len(), 1);
    assert_eq!(
        read.first().and_then(|(_, r)| r.context_window),
        Some(1_000_000)
    );
    remove_session(&path(state, "s1"));
    assert_eq!(fs::read_dir(state)?.count(), 0);
    Ok(())
}

#[test]
fn sweeping_removes_only_size_files_without_a_session() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state = dir.path();
    let record = Record {
        session_id: "live".to_owned(),
        ..Record::default()
    };
    write(state, &record)?;
    write_window(state, "live", 200_000)?;
    write_window(state, "gone", 200_000)?;
    sweep_windows(state);
    assert_eq!(read_window(state, "live"), Some(200_000));
    assert!(!state.join("gone.window").exists());
    Ok(())
}

#[test]
fn an_unusable_window_size_counts_as_none() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("a.window"), "0")?;
    fs::write(dir.path().join("b.window"), "big")?;
    assert_eq!(read_window(dir.path(), "a"), None);
    assert_eq!(read_window(dir.path(), "b"), None);
    assert_eq!(read_window(dir.path(), "missing"), None);
    Ok(())
}

#[test]
fn only_the_first_claim_of_a_status_wins_until_the_session_leaves_it() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state = dir.path();
    assert!(claim_alert(state, "s1", Status::Waiting));
    assert!(!claim_alert(state, "s1", Status::Waiting));
    clear_alerts(state, "s1", Status::Waiting);
    assert!(!claim_alert(state, "s1", Status::Waiting));
    assert!(claim_alert(state, "s1", Status::Done));
    clear_alerts(state, "s1", Status::Running);
    assert!(claim_alert(state, "s1", Status::Waiting));
    remove_session(&path(state, "s1"));
    assert_eq!(fs::read_dir(state)?.count(), 0);
    Ok(())
}

#[test]
fn removing_a_session_removes_its_lock() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state = dir.path();
    let record = Record {
        session_id: "s1".to_owned(),
        ..Record::default()
    };
    write(state, &record)?;
    fs::write(state.join("s1.lock"), "1 2")?;
    remove_session(&path(state, "s1"));
    assert_eq!(fs::read_dir(state)?.count(), 0);
    Ok(())
}
