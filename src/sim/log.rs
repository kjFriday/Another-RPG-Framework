//! [`Log`]: a bounded history of messages for the on-screen log.

use std::collections::VecDeque;

/// Keeps the newest `capacity` messages, discarding the oldest.
///
/// Backed by a `VecDeque` (a ring buffer), so removing the oldest
/// message is O(1). A plain `Vec` would shift every remaining element
/// left on each removal from the front.
#[derive(Debug)]
pub struct Log {
	lines: VecDeque<String>,
	capacity: usize,
}

impl Log {
	pub fn with_capacity(capacity: usize) -> Self {
		let lines = VecDeque::with_capacity(capacity);
		Self { lines, capacity }
	}

	/// Appends a message. Accepts anything convertible into a
	/// `String`, so both `"text"` and `format!(...)` work.
	pub fn push(&mut self, msg: impl Into<String>) {
		self.lines.push_back(msg.into());
		while self.lines.len() > self.capacity {
			self.lines.pop_front();
		}
	}

	/// Messages oldest to newest. Call `.rev()` for newest first,
	/// which the UI does to fill the log box from the bottom up.
	pub fn iter(&self) -> impl DoubleEndedIterator<Item = &str> {
		self.lines.iter().map(String::as_str)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn drops_oldest_past_capacity() {
		let mut log = Log::with_capacity(2);
		log.push("a");
		log.push("b");
		log.push("c");
		assert_eq!(log.iter().collect::<Vec<_>>(), ["b", "c"]);
	}
}
