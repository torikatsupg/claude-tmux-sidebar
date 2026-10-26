use std::cell::Cell;
use std::fs;
use std::process;

use super::*;

fn at(len: u64) -> Stamp {
    Stamp {
        len,
        modified: None,
    }
}

fn usage(context: u8) -> Latest {
    Latest {
        branch: "main".to_owned(),
        context: Some(context),
    }
}

#[test]
fn reads_a_transcript_again_only_once_it_changed() {
    let reads = Cell::new(0_u8);
    let read = |_: &Path| {
        reads.set(reads.get() + 1);
        Some(usage(reads.get()))
    };
    let a = || PathBuf::from("a");
    let cache = Transcripts::default().renew([(a(), Some(at(1)))], read);
    let cache = cache.renew([(a(), Some(at(1)))], read);
    assert_eq!((cache.latest("a"), reads.get()), (Some(usage(1)), 1));
    let cache = cache.renew([(a(), Some(at(2)))], read);
    assert_eq!((cache.latest("a"), reads.get()), (Some(usage(2)), 2));
}

#[test]
fn forgets_transcripts_that_are_gone_or_no_longer_asked_for() {
    let cache = Transcripts::default().renew(
        [
            (PathBuf::from("a"), Some(at(1))),
            (PathBuf::from("b"), Some(at(1))),
        ],
        |_| Some(usage(5)),
    );
    let cache = cache.renew([(PathBuf::from("a"), None)], |_| {
        unreachable!("a missing file is not read")
    });
    assert_eq!((cache.latest("a"), cache.latest("b")), (None, None));
    assert!(cache.0.is_empty());
}

#[test]
fn refresh_follows_the_file_on_disk() {
    let path = std::env::temp_dir().join(format!("cts-transcript-{}.jsonl", process::id()));
    let entry =
        r#"{"type":"assistant","gitBranch":"main","message":{"usage":{"input_tokens":100000}}}"#;
    let _ = fs::write(&path, format!("{entry}\n"));
    let key = path.to_string_lossy().into_owned();
    let cache = Transcripts::default().refresh([path.as_path()]);
    assert_eq!(cache.latest(&key).map(|l| l.context), Some(Some(50)));
    let compacted = r#"{"type":"system","subtype":"compact_boundary","gitBranch":"main"}"#;
    let _ = fs::write(&path, format!("{entry}\n{compacted}\n"));
    let after = cache.refresh([path.as_path()]);
    let _ = fs::remove_file(&path);
    assert_eq!(after.latest(&key).map(|l| l.context), Some(None));
    assert_eq!(after.refresh([path.as_path()]).latest(&key), None);
}
