use std::sync::Arc;

use net::auth::OAuth;
use {App, ResponseGenFn, Scopes};

use failure::Error;

impl App {
	/// Authorize this app as a script
	/// # Arguments
	/// * `id` - The app id registered on Reddit
	/// * `secret` - The app secret registered on Reddit
	/// * `username` - The username of the user to authorize as
	/// * `password` - The password of the user to authorize as
	pub fn authorize_script(&mut self, id: &str, secret: &str, username: &str, password: &str) -> Result<(), Error> {
		let auth = OAuth::create_script(&self.conn, id, secret, username, password)?;
		self.conn.auth = Some(auth);
		Ok(())
	}

	/// Authorize this app as an installed app
	/// # Arguments
	/// * `conn` - A reference to the connection to authorize
	/// * `id` - The app id registered on Reddit
	/// * `redirect` - The app redirect URI registered on Reddit
	/// * `response_gen` - An optional function that generates a hyper Response to give to the user
	/// based on the result of the authorization attempt. The signature is `(Result<String, InstalledAppError) -> Result<Response, Response>`.
	/// The result passed in is either Ok with the code recieved, or Err with the error that occurred.
	/// The value returned should usually be an Ok(Response), but you can return Err(Response) to indicate
	/// that an error occurred within the function.
	/// * `scopes` - A reference to a Scopes instance representing the capabilites you are requesting
	/// as an installed app.
	pub fn authorize_installed_app<I: Into<Option<Arc<ResponseGenFn>>>>(&mut self, id: &str, redirect: &str, response_gen: I, scopes: &Scopes) -> Result<(), Error> {
		let auth = OAuth::create_installed_app(&self.conn, id, redirect, response_gen, scopes)?;
		self.conn.auth = Some(auth);
		Ok(())
	}
}
