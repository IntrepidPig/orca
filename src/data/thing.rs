use json;
use errors::*;

pub trait Thing {
    fn from_value(&json::Value) -> Result<Self>
    where
        Self: Sized;
    fn get_json(&self) -> &json::Value;
}
