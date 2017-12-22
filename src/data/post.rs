use json::Value;
use data::Thing;
use errors::*;
use failure::Error;

#[derive(Debug)]
pub struct Post {
	pub raw: Value,
}

impl Post {}

impl Thing for Post {
	fn from_value(data: &Value) -> Result<Post, Error> {
		Ok(Post { raw: data.clone() })
	}

	fn get_json(&self) -> &Value {
		&self.raw
	}
}
