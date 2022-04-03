use gdnative::prelude::*;

pub struct Pipe {
	start_id: i64,
	end_id: i64,
}

impl Pipe {
	pub fn new(start_id: i64, end_id: i64) -> Self {
		Self {
			// canonical order
			start_id: i64::min(start_id, end_id),
			end_id: i64::max(end_id, start_id),
		}
	}

	pub fn start_id(&self) -> i64 {
		self.start_id
	}

	pub fn end_id(&self) -> i64 {
		self.end_id
	}
}
