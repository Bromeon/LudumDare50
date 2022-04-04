#[derive(Debug, Copy, Clone)]
pub struct Pipe {
	pipe_id: i64,
	start_id: i64,
	end_id: i64,
}

impl Pipe {
	pub fn new(pipe_id: i64, start_id: i64, end_id: i64) -> Self {
		assert_ne!(start_id, pipe_id);
		assert_ne!(start_id, end_id);
		Self {
			pipe_id,
			// canonical order
			start_id: i64::min(start_id, end_id),
			end_id: i64::max(end_id, start_id),
		}
	}

	pub fn pipe_node_id(&self) -> i64 {
		self.pipe_id
	}

	pub fn start_node_id(&self) -> i64 {
		self.start_id
	}

	pub fn end_node_id(&self) -> i64 {
		self.end_id
	}
}
