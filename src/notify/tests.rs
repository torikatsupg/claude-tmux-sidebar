use super::place::{Client, Place};
use super::*;

mod worth;
use crate::i18n;

#[test]
fn off_unless_set_to_a_known_mode() {
    assert_eq!(Mode::parse(None), Mode::Off);
    assert_eq!(Mode::parse(Some("off")), Mode::Off);
    assert_eq!(Mode::parse(Some("bogus")), Mode::Off);
    assert_eq!(Mode::parse(Some("tmux")), Mode::Tmux);
    assert_eq!(Mode::parse(Some("os")), Mode::Os);
    assert_eq!(Mode::parse(Some("all")), Mode::All);
    let channels = |m: Mode| (m.tmux(), m.os());
    assert_eq!(channels(Mode::Off), (false, false));
    assert_eq!(channels(Mode::Tmux), (true, false));
    assert_eq!(channels(Mode::Os), (false, true));
    assert_eq!(channels(Mode::All), (true, true));
}

#[test]
fn reads_the_mode_sidebar_tmux_recorded() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    assert_eq!(Mode::read(dir.path()), Mode::Off);
    fs::write(dir.path().join("notify"), "all\n")?;
    assert_eq!(Mode::read(dir.path()), Mode::All);
    Ok(())
}

fn place(window: &str, windows: u32, zoomed: bool, active: bool) -> Place {
    Place {
        session: "api".to_owned(),
        window_index: 2,
        windows,
        window: window.to_owned(),
        zoomed,
        active,
        tty: "/dev/ttys003".to_owned(),
    }
}

#[test]
fn reads_where_the_pane_is_and_what_the_clients_show() {
    assert_eq!(
        place::parse_place("api\t2\t3\t@4\t0\t1\t/dev/ttys003"),
        Some(place("@4", 3, false, true))
    );
    assert_eq!(place::parse_place(""), None);
    let clients = place::parse_clients("/dev/ttys001\t100\t@4\n/dev/ttys002\t300\t@7\nbroken\n");
    assert_eq!(
        place::watcher(&clients).map(|c| c.name.as_str()),
        Some("/dev/ttys002")
    );
}

fn client(window: &str) -> Client {
    Client {
        name: "/dev/ttys001".to_owned(),
        activity: 1,
        window: window.to_owned(),
    }
}

#[test]
fn the_pane_is_seen_when_the_active_client_shows_its_window() {
    let shown = client("@4");
    assert!(place::seen(&place("@4", 1, false, false), Some(&shown)));
    assert!(place::seen(&place("@4", 1, true, true), Some(&shown)));
    assert!(!place::seen(&place("@4", 1, true, false), Some(&shown)));
    assert!(!place::seen(
        &place("@4", 1, false, true),
        Some(&client("@5"))
    ));
    assert!(!place::seen(&place("@4", 1, false, true), None));
}

#[test]
fn the_message_names_the_session_and_its_status() {
    let en = i18n::messages(None);
    let ja = i18n::messages(Some("ja"));
    assert_eq!(
        text(Status::Waiting, &place("@4", 1, false, true), en),
        "◆ api waiting for you"
    );
    assert_eq!(
        text(Status::Done, &place("@4", 3, false, true), en),
        "✓ api:2 done"
    );
    assert_eq!(
        text(Status::Waiting, &place("@4", 1, false, true), ja),
        "◆ api 承認待ち"
    );
}

#[test]
fn tmux_does_not_expand_formats_in_the_message() {
    assert_eq!(tmux_text("✓ #(rm x):2 done"), "✓ ##(rm x):2 done");
}

#[test]
fn the_os_notification_command_takes_the_message_as_an_argument() {
    let (program, args) = os_command("◆ a\"b waiting for you", true);
    assert_eq!(program, "osascript");
    assert_eq!(
        args.last().map(String::as_str),
        Some("◆ a\"b waiting for you")
    );
    assert!(
        args.iter()
            .all(|a| !a.contains("a\"b") || a == "◆ a\"b waiting for you")
    );
    let (program, args) = os_command("✓ a<b>&c done", false);
    assert_eq!(program, "notify-send");
    assert_eq!(args, ["Claude Code", "✓ a&lt;b&gt;&amp;c done"]);
}
