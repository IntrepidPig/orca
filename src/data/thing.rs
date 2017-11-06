use json;

pub trait Thing {
	fn get_json(&self) -> &json::Value;
}