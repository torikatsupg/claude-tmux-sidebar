//! `sidebar` / `popup-ui`: the interactive list. The event loop is a fold over steps; each step draws,
//! reads input, applies the pure [`update`](update::update) and runs the resulting effects.

mod effect;
mod env;
pub mod headings;
pub mod indent;
mod input;
pub mod label;
pub mod preview;
pub mod rows;
pub mod selection;
pub mod snapshot;
pub mod theme;
pub mod update;
mod view;

use std::io;
use std::ops::ControlFlow;
use std::time::{Duration, Instant, SystemTime};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use ratatui::crossterm::execute;

use self::{effect::Next, env::Env};
use crate::context::{self, Context};
use crate::session::store;
use crate::session::transcript::cache::Transcripts;
use crate::tmux::pane::PaneId;
use update::{Effect, Model, Msg};

/// Input wait per step; also the spinner speed.
const FRAME: Duration = Duration::from_millis(150);
/// State files are re-read at least this often, and immediately when the directory changes.
const RELOAD_EVERY: Duration = Duration::from_millis(500);

/// Where the list runs.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    /// A sidebar pane (its id comes from `TMUX_PANE`).
    Sidebar,
    /// A popup opened from `origin`.
    Popup { origin: Option<PaneId> },
}

/// The loop state carried from step to step.
struct Loop {
    model: Model,
    loaded_at: Instant,
    mtime: Option<SystemTime>,
    transcripts: Transcripts,
}

pub fn run(ctx: &Context, mode: Mode) -> io::Result<()> {
    let env = Env::new(ctx, mode);
    ratatui::run(|terminal| {
        execute!(io::stdout(), EnableMouseCapture, EnableFocusChange)?;
        let result = event_loop(terminal, &env);
        execute!(io::stdout(), DisableMouseCapture, DisableFocusChange)?;
        result
    })
}

type Flow<T> = ControlFlow<io::Result<()>, T>;

fn event_loop(terminal: &mut DefaultTerminal, env: &Env<'_>) -> io::Result<()> {
    let height = terminal.size()?.height;
    let model = Model {
        popup: env.scope.popup,
        help: env.help,
        label: env.scope.label,
        indent: env.indent,
        theme: env.theme,
        preview: env.preview,
        home: env.home.clone(),
        height,
        now: context::now(),
        ..Model::default()
    };
    let start = Loop {
        model,
        loaded_at: Instant::now(),
        mtime: None,
        transcripts: Transcripts::default(),
    };
    steps(terminal, env, start).break_value().unwrap_or(Ok(()))
}

fn steps(terminal: &mut DefaultTerminal, env: &Env<'_>, start: Loop) -> Flow<Loop> {
    let first = reload(env, start)?;
    std::iter::repeat(()).try_fold(first, |state, ()| step(terminal, env, state))
}

/// Draws, waits briefly for input, applies the messages, and reloads when due.
fn step(terminal: &mut DefaultTerminal, env: &Env<'_>, state: Loop) -> Flow<Loop> {
    io_flow(terminal.draw(|frame| view::render(&state.model, env.messages, frame)))?;
    let input = io_flow(read_input(env))?;
    let answers = env.answered.try_iter();
    let messages = input
        .into_iter()
        .chain(answers)
        .chain([tick(env.started)])
        .collect();
    let (state, forced) = apply(env, state, messages)?;
    if forced || due(env, &state) {
        reload(env, state)
    } else {
        ControlFlow::Continue(state)
    }
}

fn tick(started: Instant) -> Msg {
    let frame = started.elapsed().as_millis() / FRAME.as_millis();
    Msg::Tick {
        now: context::now(),
        frame: usize::try_from(frame).unwrap_or_default(),
    }
}

fn io_flow<T>(result: io::Result<T>) -> Flow<T> {
    result.map_or_else(|e| ControlFlow::Break(Err(e)), ControlFlow::Continue)
}

fn read_input(env: &Env<'_>) -> io::Result<Vec<Msg>> {
    if event::poll(FRAME)? {
        let message = input::message(&event::read()?, env.scope.popup, env.popup_key());
        Ok(message.into_iter().collect())
    } else {
        Ok(Vec::new())
    }
}

/// Folds the messages through `update`, then runs the effects. The flag asks for a reload.
fn apply(env: &Env<'_>, state: Loop, messages: Vec<Msg>) -> Flow<(Loop, bool)> {
    let (model, effects) =
        messages
            .into_iter()
            .fold((state.model, Vec::new()), |(model, done), msg| {
                let (model, effects) = update::update(model, msg);
                (model, done.into_iter().chain(effects).collect())
            });
    let (model, reload) = settle(env, model, &effects)?;
    ControlFlow::Continue((Loop { model, ..state }, reload))
}

/// Runs the effects. The flag asks for a reload.
fn settle(env: &Env<'_>, model: Model, effects: &[Effect]) -> Flow<(Model, bool)> {
    match effect::perform(env.ctx, env.scope.me, &env.git, effects) {
        Next::Stop => ControlFlow::Break(Ok(())),
        Next::Continue { reload } => ControlFlow::Continue((model, reload)),
    }
}

fn due(env: &Env<'_>, state: &Loop) -> bool {
    state.loaded_at.elapsed() >= RELOAD_EVERY || store::mtime(&env.ctx.state_dir) != state.mtime
}

fn reload(env: &Env<'_>, state: Loop) -> Flow<Loop> {
    let (snapshot, transcripts) = snapshot::load(env.ctx, &env.scope, state.transcripts);
    let mtime = store::mtime(&env.ctx.state_dir);
    let (model, effects) = update::update(state.model, Msg::Loaded(snapshot));
    let (model, _) = settle(env, model, &effects)?;
    ControlFlow::Continue(Loop {
        model,
        loaded_at: Instant::now(),
        mtime,
        transcripts,
    })
}
