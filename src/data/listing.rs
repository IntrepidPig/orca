use std::collections::VecDeque;

use json;
use json::Value;
use serde::de::Error;

use data::{CommentData, Comment};

#[derive(Clone)]
pub struct Listing<T> {
	pub children: VecDeque<T>
}

impl<T> Listing<T> {
	pub fn empty() -> Listing<T> {
		Listing { children: VecDeque::new() }
	}
}

impl<T> Iterator for Listing<T> {
	type Item = T;
	
	fn next(&mut self) -> Option<Self::Item> {
		self.children.pop_front()
	}
}

impl Listing<Comment> {
	pub fn from_value(listing: &Value) -> Result<Listing<Comment>, json::Error> {
		let mut children: VecDeque<Comment> = VecDeque::new();
		
		if let Some(array) = listing["data"]["children"].as_array() {
			for item in array {
				let kind = item["kind"].as_str().unwrap();
				if kind == "t1" {
					children.push_back(Comment::from_value(item).unwrap());
				} else if kind == "more" {
					for extra in item["data"]["children"].as_array().unwrap() {
						children.push_back(Comment::NotLoaded(extra.as_str().unwrap().to_string()));
					}
				}
			}
			
			Ok(Listing { children })
		} else {
			Err(json::Error::custom("Couldn't parse as array"))
		}
	}
}