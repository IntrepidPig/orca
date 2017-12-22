use json;
use errors::*;
use failure::Error;

pub trait Thing {
	fn from_value(data: &json::Value) -> Result<Self, Error>
	where
		Self: Sized;
	fn get_json(&self) -> &json::Value;
}
