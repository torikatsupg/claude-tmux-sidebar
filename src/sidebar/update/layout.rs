//! Where the list ends: above the preview and the help line.

use super::Model;
use crate::sidebar::{preview, rows};

impl Model {
    /// The screen line below the list: the preview, the help line, or the bottom edge.
    pub fn list_end(&self, height: u16) -> u16 {
        let bottom = height.saturating_sub(u16::from(self.help));
        bottom.saturating_sub(self.preview_height(height))
    }

    /// The lines of the preview below the list, its rule included. The preview shows only while the list
    /// is in front of the user; otherwise the cursor mirrors the pane they look at, which needs no preview.
    pub fn preview_height(&self, height: u16) -> u16 {
        if !self.snapshot.focused {
            return 0;
        }
        let area = height
            .saturating_sub(u16::from(self.help))
            .saturating_sub(rows::FIRST_ROW);
        preview::height(self.preview.mode, area)
    }
}
