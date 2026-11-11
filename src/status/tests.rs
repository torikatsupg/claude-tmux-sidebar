use super::*;

fn counts(waiting: usize, running: usize, done: usize) -> Counts {
    Counts {
        waiting,
        running,
        done,
    }
}

#[test]
fn counts_waiting_running_and_done_but_not_idle() {
    let states = [
        Status::Waiting,
        Status::Done,
        Status::Idle,
        Status::Waiting,
        Status::Running,
        Status::Done,
        Status::Done,
    ];
    assert_eq!(Counts::of(states), counts(2, 1, 3));
}

#[test]
fn prints_the_counts_on_one_line() {
    assert_eq!(line(counts(2, 1, 3), None), "◆2 ⠹1 ✓3");
}

#[test]
fn leaves_out_statuses_with_no_session() {
    assert_eq!(line(counts(0, 4, 0), None), "⠹4");
    assert_eq!(line(counts(1, 0, 2), None), "◆1 ✓2");
    assert_eq!(line(counts(0, 0, 0), None), "");
}

#[test]
fn colors_each_count_with_a_tmux_style() {
    let colors =
        Colors::from_options(|name| (name == "@cts_color_done").then(|| "#00ffff".to_owned()));
    assert_eq!(
        line(counts(1, 1, 1), Some(&colors)),
        "#[fg=yellow]◆1#[fg=default] #[fg=green]⠹1#[fg=default] #[fg=#00ffff]✓1#[fg=default]"
    );
    assert_eq!(line(counts(0, 0, 0), Some(&colors)), "");
}

#[test]
fn a_color_that_would_break_the_style_keeps_its_default() {
    for value in ["red]#[bg=blue", "lightred", "#12", "coral"] {
        let colors = Colors::from_options(|_| Some(value.to_owned()));
        assert_eq!(colors, Colors::default(), "{value}");
    }
    let colors = Colors::from_options(|_| Some("brightRed".to_owned()));
    assert_eq!(colors.done, "brightRed");
}
