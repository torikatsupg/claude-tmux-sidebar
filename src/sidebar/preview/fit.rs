//! Fitting the panel's sections into the preview's height.

use super::panel::Item;

/// Lines that stay while they fit (`head`), and lines that take what room is left (`body`), from its end
/// when `tail`.
#[derive(Default)]
pub struct Section {
    pub head: Vec<Item>,
    pub body: Vec<Item>,
    pub tail: bool,
}

/// The heads in order while they fit, then the bodies in order with the room left; shown in section order.
pub fn fit(sections: &[Section], room: usize) -> Vec<Item> {
    let (kept, used) = sections
        .iter()
        .map(|s| s.head.len())
        .scan(0, |used, len| {
            *used += len;
            Some(*used)
        })
        .take_while(|&used| used <= room)
        .enumerate()
        .last()
        .map_or((0, 0), |(i, used)| (i + 1, used));
    let (_, shown) =
        sections
            .iter()
            .take(kept)
            .fold((room - used, Vec::new()), |(left, shown), section| {
                let (lines, take) = shown_of(section, left);
                (left - take, shown.into_iter().chain(lines).collect())
            });
    shown
}

/// A section's head and as much of its body as `left` allows, and how much of the body that is.
fn shown_of(section: &Section, left: usize) -> (Vec<Item>, usize) {
    let take = section.body.len().min(left);
    let skip = if section.tail {
        section.body.len() - take
    } else {
        0
    };
    let body = section.body.iter().skip(skip).take(take);
    (section.head.iter().chain(body).cloned().collect(), take)
}
