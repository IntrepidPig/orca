use std::collections::HashMap;

use failure::Error;
use hyper::{Request, Method};

use App;
use net::body_from_map;

impl App {
	/// Send a private message to a user
	/// # Arguments
	/// * `to` - Name of the user to send a message to
	/// * `subject` - Subject of the message
	/// * `body` - Body of the message
	pub fn message(&self, to: &str, subject: &str, body: &str) -> Result<(), Error> {
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("to", to);
		params.insert("subject", subject);
		params.insert("text", body);
		
		let mut req = Request::new(
			Method::Post,
			"https://oauth.reddit.com/api/compose/.json".parse()?,
		);
		req.set_body(body_from_map(&params));
		
		match self.conn.run_auth_request(req) {
			Ok(_) => Ok(()),
			Err(e) => Err(e),
		}
	}
}