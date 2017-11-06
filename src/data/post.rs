use json::Value;
use data::Thing;

pub struct Post {
	pub raw: Value,
}

impl Post {
	pub fn from_json(data: Value) -> Post {
		Post {
			raw: data
		}
	}
}

impl Thing for Post {
	fn get_json(&self) -> &Value {
		&self.raw
	}
}