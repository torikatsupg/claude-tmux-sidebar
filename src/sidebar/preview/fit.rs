//! Fitting the panel's sections into the preview's height.

use super::panel::Item;

/// Lines that stay while they fit (`head`), and lines that take what room is left (`body`), from its end
/// when `tail`. `min_body` lines of the body go before any head; a `first` head goes before the others.
#[derive(Default)]
pub struct Section {
    pub head: Vec<Item>,
    pub body: Vec<Item>,
    pub tail: bool,
    pub min_body: usize,
    pub first: bool,
}

/// The reserved body lines, then the heads (`first` ones, then the rest, each kept whole if it fits), then
/// the bodies in order with the room left. Shown in section order; rows go with their head.
pub fn fit(sections: &[Section], room: usize) -> Vec<Item> {
    let reserved = reserve(sections, room);
    let left = room - reserved.iter().sum::<usize>();
    let (kept, left) = heads(sections, left);
    let (_, shown) = sections
        .iter()
        .zip(reserved)
        .zip(kept)
        .filter(|(_, kept)| *kept)
        .fold(
            (left, Vec::new()),
            |(left, shown), ((section, reserved), _)| {
                let extra = section.body.len().saturating_sub(reserved).min(left);
                let lines = shown_of(section, reserved + extra);
                (left - extra, shown.into_iter().chain(lines).collect())
            },
        );
    shown
}

/// Each section's reserved body lines, in order, within `room`.
fn reserve(sections: &[Section], room: usize) -> Vec<usize> {
    let (_, reserved) = sections.iter().fold((room, Vec::new()), |(left, done), s| {
        let take = s.min_body.min(s.body.len()).min(left);
        (left - take, done.into_iter().chain([take]).collect())
    });
    reserved
}

/// Which heads fit in `room`, and the room left after them.
fn heads(sections: &[Section], room: usize) -> (Vec<bool>, usize) {
    let order = sections
        .iter()
        .enumerate()
        .filter(|(_, s)| s.first)
        .chain(sections.iter().enumerate().filter(|(_, s)| !s.first));
    let (left, kept) = order.fold((room, Vec::new()), |(left, kept), (i, s)| {
        if s.head.len() <= left {
            (left - s.head.len(), kept.into_iter().chain([i]).collect())
        } else {
            (left, kept)
        }
    });
    (
        (0..sections.len()).map(|i| kept.contains(&i)).collect(),
        left,
    )
}

/// A section's head and `take` lines of its body.
fn shown_of(section: &Section, take: usize) -> Vec<Item> {
    let skip = if section.tail {
        section.body.len() - take
    } else {
        0
    };
    let body = section.body.iter().skip(skip).take(take);
    section.head.iter().chain(body).cloned().collect()
}
