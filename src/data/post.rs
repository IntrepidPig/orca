use json::Value;
use data::Thing;
use errors::*;

#[derive(Debug)]
pub struct Post {
    pub raw: Value,
}

impl Post {}

impl Thing for Post {
    fn from_value(data: &Value) -> Result<Post> {
        Ok(Post { raw: data.clone() })
    }

    fn get_json(&self) -> &Value {
        &self.raw
    }
}
