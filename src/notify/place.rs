//! Where the pane is, and whether the user is looking at it.

/// The `display -p -t PANE` format that [`parse_place`] reads.
pub const PLACE_FORMAT: &str = "#{session_name}\t#{window_index}\t#{session_windows}\t#{window_id}\t\
                                #{window_zoomed_flag}\t#{pane_active}\t#{pane_tty}";

/// The `list-clients -F` format that [`parse_clients`] reads.
pub const CLIENT_FORMAT: &str = "#{client_name}\t#{client_activity}\t#{window_id}";

/// The pane's tmux session and window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub session: String,
    pub window_index: u32,
    /// How many windows the session has.
    pub windows: u32,
    pub window: String,
    pub zoomed: bool,
    /// The pane is its window's active pane.
    pub active: bool,
    pub tty: String,
}

/// An attached client and the window it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub name: String,
    /// When the user last typed in it.
    pub activity: u64,
    pub window: String,
}

pub fn parse_place(output: &str) -> Option<Place> {
    let fields: Vec<&str> = output.trim_end_matches('\n').split('\t').collect();
    let [session, index, windows, window, zoomed, active, tty] = fields.as_slice() else {
        return None;
    };
    Some(Place {
        session: (*session).to_owned(),
        window_index: index.parse().ok()?,
        windows: windows.parse().ok()?,
        window: (*window).to_owned(),
        zoomed: *zoomed == "1",
        active: *active == "1",
        tty: (*tty).to_owned(),
    })
}

/// Parses `list-clients -F CLIENT_FORMAT` output, skipping malformed lines.
pub fn parse_clients(output: &str) -> Vec<Client> {
    output
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            let [name, activity, window] = fields.as_slice() else {
                return None;
            };
            Some(Client {
                name: (*name).to_owned(),
                activity: activity.parse().ok()?,
                window: (*window).to_owned(),
            })
        })
        .collect()
}

/// The client the user is using now: the one with the latest input.
pub fn watcher(clients: &[Client]) -> Option<&Client> {
    clients.iter().max_by_key(|c| c.activity)
}

/// Whether `client` shows the pane: its window is on screen and not zoomed into another pane.
pub fn seen(place: &Place, client: Option<&Client>) -> bool {
    client.is_some_and(|c| c.window == place.window) && (!place.zoomed || place.active)
}
