use json;
use errors::*;

pub trait Thing {
	fn from_value(data: &json::Value) -> Result<Self, RedditError>
	where
		Self: Sized;
	fn get_json(&self) -> &json::Value;
}
