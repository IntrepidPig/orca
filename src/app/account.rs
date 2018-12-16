use failure::Error;
use hyper::{Body, Request};
use json::Value;

use App;

impl App {
	/// Get info of the user currently authorized
	///
	/// Note: requires connection to be authorized
	/// # Returns
	/// A result with the json value of the user data
	pub fn get_self(&self) -> Result<Value, Error> {
		let req = Request::get("https://oauth.reddit.com/api/v1/me/.json").body(Body::empty()).unwrap();

		self.conn.run_auth_request(req)
	}
}
