//! tmux-side commands: open, close and maintain the sidebar pane in every window.

pub mod layout;

use layout::{LAYOUT_OPTION, PANES_OPTION, Saved};

use crate::context::Context;
use crate::text::shell_quote;
use crate::tmux::{
    self,
    pane::{self, Pane, PaneId, WindowId},
};

const DEFAULT_WIDTH: u16 = 34;
const ENABLED_OPTION: &str = "@cts_enabled";
const SIDEBAR_OPTION: &str = "@cts_sidebar";
/// The popup is a little wider than the sidebar to fit its border.
const POPUP_EXTRA_WIDTH: u16 = 6;

fn width() -> u16 {
    tmux::global_option("@cts_width")
        .and_then(|w| w.parse().ok())
        .unwrap_or(DEFAULT_WIDTH)
}

fn enabled() -> bool {
    tmux::global_option(ENABLED_OPTION).as_deref() == Some("on")
}

/// Opens the sidebar, focuses it if it is open, and closes it if it already has the focus.
pub fn toggle(ctx: &Context, origin: Option<PaneId>) {
    if !enabled() {
        return on(ctx, origin);
    }
    match origin.and_then(find_pane) {
        Some(pane) if !pane.sidebar => {
            ensure(ctx, Some(pane.window_id));
            focus_sidebar(pane.window_id);
        }
        _ => off(),
    }
}

/// Opens the sidebar everywhere and focuses the one next to `origin`.
pub fn on(ctx: &Context, origin: Option<PaneId>) {
    tmux::run(&["set", "-g", ENABLED_OPTION, "on"]);
    for window in windows() {
        ensure(ctx, Some(window));
    }
    if let Some(pane) = origin.and_then(find_pane) {
        focus_sidebar(pane.window_id);
    }
}

fn find_pane(id: PaneId) -> Option<Pane> {
    tmux::panes().into_iter().find(|p| p.id == id)
}

fn focus_sidebar(window: WindowId) {
    if let Some(sidebar) = pane::sidebar_of(&tmux::window_panes(window), window) {
        tmux::run(&["select-pane", "-t", &sidebar.id.to_string()]);
    }
}

pub fn off() {
    tmux::run(&["set", "-g", ENABLED_OPTION, "off"]);
    for sidebar in tmux::panes().iter().filter(|p| p.sidebar) {
        close(sidebar);
    }
}

fn windows() -> Vec<WindowId> {
    tmux::output(&["list-windows", "-a", "-F", "#{window_id}"])
        .lines()
        .filter_map(WindowId::parse)
        .collect()
}

/// Adds the sidebar to a window if it is enabled and missing (run from tmux hooks).
pub fn ensure(ctx: &Context, window: Option<WindowId>) {
    let Some(window) = window.or_else(tmux::current_window).filter(|_| enabled()) else {
        return;
    };
    // One window switch fires several tmux hooks at once; serialize them so only one sidebar gets added.
    let lock = format!("cts-{window}");
    tmux::run(&["wait-for", "-L", &lock]);
    let panes = tmux::window_panes(window);
    let zoomed = tmux::display(&window.to_string(), "#{window_zoomed_flag}") == "1";
    if layout::needs_sidebar(&panes, zoomed) {
        save_layout(window, &panes);
        split_sidebar(ctx, window);
    }
    tmux::run(&["wait-for", "-U", &lock]);
}

/// Remembers the layout so closing the sidebar can put the panes back as they were.
fn save_layout(window: WindowId, panes: &[Pane]) {
    let layout = tmux::display(&window.to_string(), "#{window_layout}");
    set_window_option(window, LAYOUT_OPTION, &layout);
    set_window_option(
        window,
        PANES_OPTION,
        &layout::encode(&layout::real_panes(panes)),
    );
}

fn split_sidebar(ctx: &Context, window: WindowId) {
    let command = format!("exec {} sidebar", shell_quote(&ctx.exe.to_string_lossy()));
    let (width, target) = (width().to_string(), window.to_string());
    let split = [
        "split-window",
        "-hbfd",
        "-l",
        &width,
        "-t",
        &target,
        "-P",
        "-F",
        "#{pane_id}",
        &command,
    ];
    if let Some(sidebar) = PaneId::parse(tmux::output(&split).trim()) {
        let id = sidebar.to_string();
        tmux::run(&["set", "-p", "-t", &id, SIDEBAR_OPTION, "1"]);
        tmux::run(&["set", "-p", "-t", &id, "remain-on-exit", "off"]);
    }
}

/// Closes one sidebar, handing the focus back to the pane the user was in and restoring the layout.
fn close(sidebar: &Pane) {
    let id = sidebar.id.to_string();
    if sidebar.active {
        tmux::run(&["last-pane", "-t", &id]);
    }
    tmux::run(&["kill-pane", "-t", &id]);
    restore_layout(sidebar.window_id);
}

fn restore_layout(window: WindowId) {
    let saved = tmux::window_option(window, LAYOUT_OPTION).map(|layout| Saved {
        layout,
        panes: layout::decode(&tmux::window_option(window, PANES_OPTION).unwrap_or_default()),
    });
    let target = window.to_string();
    if let Some(layout) = layout::restorable(saved.as_ref(), &tmux::window_panes(window)) {
        tmux::run(&["select-layout", "-t", &target, layout]);
    }
    for option in [LAYOUT_OPTION, PANES_OPTION] {
        tmux::run(&["set", "-wu", "-t", &target, option]);
    }
}

fn set_window_option(window: WindowId, name: &str, value: &str) {
    tmux::run(&["set", "-w", "-t", &window.to_string(), name, value]);
}

/// Restores the sidebar width after a layout change (run from tmux hooks).
pub fn fix_width(window: Option<WindowId>) {
    let Some(window) = window.or_else(tmux::current_window) else {
        return;
    };
    let wanted = width().to_string();
    let sidebar = pane::sidebar_of(&tmux::window_panes(window), window)
        .map(|s| s.id.to_string())
        .filter(|id| tmux::display(id, "#{pane_width}") != wanted);
    if let Some(id) = sidebar {
        tmux::run(&["resize-pane", "-t", &id, "-x", &wanted]);
    }
}

/// Shows the list in a popup instead of the sidebar.
pub fn popup(ctx: &Context, origin: Option<PaneId>) {
    let origin = origin.map(|o| o.to_string()).unwrap_or_default();
    let command = format!(
        "{} popup-ui {}",
        shell_quote(&ctx.exe.to_string_lossy()),
        shell_quote(&origin)
    );
    let width = (width() + POPUP_EXTRA_WIDTH).to_string();
    tmux::run(&[
        "display-popup",
        "-E",
        "-w",
        &width,
        "-h",
        "80%",
        "-T",
        " Claude ",
        &command,
    ]);
}
