use std::collections::HashMap;

use failure::Error;
use hyper::Request;
use url::form_urlencoded;

use net::body_from_map;
use App;

impl App {
	/// Send a private message to a user
	/// # Arguments
	/// * `to` - Name of the user to send a message to
	/// * `subject` - Subject of the message
	/// * `body` - Body of the message
	pub fn message(&self, to: &str, subject: &str, body: &str) -> Result<(), Error> {
		let subject: String = form_urlencoded::byte_serialize(subject.as_bytes()).collect();
		let body: String = form_urlencoded::byte_serialize(body.as_bytes()).collect();
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("to", to);
		params.insert("subject", &subject);
		params.insert("text", &body);

		let req = Request::post("https://oauth.reddit.com/api/compose/.json").body(body_from_map(&params)).unwrap();

		match self.conn.run_auth_request(req) {
			Ok(_) => Ok(()),
			Err(e) => Err(e),
		}
	}
}
