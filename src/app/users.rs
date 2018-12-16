use failure::Error;
use hyper::{Body, Request};
use json::Value;

use App;

impl App {
	/// Gets information about a user that is not currently authorized
	/// # Arguments
	/// * `name` - username of the user to query
	/// # Returns
	/// A json value containing the user info
	pub fn get_user(&self, name: &str) -> Result<Value, Error> {
		let req = Request::get(format!("https://www.reddit.com/user/{}/about/.json", name)).body(Body::empty()).unwrap();

		self.conn.run_request(req)
	}
}
