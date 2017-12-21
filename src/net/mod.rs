/// Contains all functionality for OAuth and logins
pub mod auth;
/// Reddit errors
pub mod error;

use std::io::Read;
use std::time::{Duration, Instant};
use std::thread;
use std::cell::Cell;

use json;
use json::Value;
use http::{Client, Method, Request, Url};
use http::header::{Authorization, Bearer, UserAgent, Basic};

use errors::{BadRequest, RedditError, Forbidden};
use self::auth::{OAuth, OauthApp};

use failure::{Fail, Error, err_msg};

#[derive(Copy, Clone)]
pub enum LimitMethod {
	Steady,
	Burst,
}

/// A connection holder to reddit. Holds authorization info if provided
pub struct Connection {
	/// Authorization info (optional, but required for sending authorized requests)
	pub auth: Option<auth::OAuth>,
	/// User agent for the client
	pub useragent: UserAgent,
	/// HTTP client
	pub client: Client,
	/// How to ratelimit (burst or steady)
	pub limit: Cell<LimitMethod>,
	/// Requests sent in the past ratelimit period
	reqs: Cell<i32>,
	/// Requests remaining
	remaining: Cell<Option<i32>>,
	/// Time when request amount will reset
	reset_time: Cell<Instant>,
}

impl Connection {
	pub fn new(appname: &str, appversion: &str, appauthor: &str) -> Result<Connection, Error> {
		let useragent = UserAgent::new(format!(
			"linux:{}:{} (by {})",
			appname,
			appversion,
			appauthor
		));
		Ok(Connection {
			auth: None,
			useragent,
			client: Client::new()?,
			limit: Cell::new(LimitMethod::Steady),
			reqs: Cell::new(0),
			remaining: Cell::new(None),
			reset_time: Cell::new(Instant::now()),
		})
	}

	/// Send a request to reddit
	pub fn run_request(&self, mut req: Request) -> Result<Value, RedditError> {
		// Ratelimit based on method chosen type
		match self.limit.get() {
			LimitMethod::Steady => {
				// Check if we have a remaining limit
				if let Some(remaining) = self.remaining.get() {
					// If the reset time is in the future
					if Instant::now() < self.reset_time.get() {
						// Sleep for the amount of time until reset divided by how many requests we have for steady sending
						thread::sleep(
							(self.reset_time.get() - Instant::now())
								.checked_div(remaining as u32)
								.unwrap(),
						);
					}
					// Else we must have already passed reset time and we will get a new one after this request
				}
			}
			LimitMethod::Burst => {
				// Check if we have a remaining limit
				if let Some(remaining) = self.remaining.get() {
					// If we have none remaining and we haven't passed the request limit, sleep till we do
					if remaining <= 0 && self.reset_time.get() > Instant::now() {
						thread::sleep(Instant::now() - self.reset_time.get());
					}
				}
			}
		};


		req.headers_mut().set(self.useragent.clone());
		// Execute the request!
		let mut response = match self.client.execute(req) {
			Ok(resp) => resp,
			Err(_) => return Err(RedditError::BadRequest),
		};
		let mut out = String::new();
		response.read_to_string(&mut out).unwrap();

		// Update values from response ratelimiting headers
		if let Some(reqs_used) = response.headers().get_raw("x-ratelimit-used") {
			let reqs_used = String::from_utf8_lossy(reqs_used.one().unwrap())
				.parse::<f32>()
				.unwrap()
				.round() as i32;
			self.reqs.set(reqs_used);
		}
		if let Some(reqs_remaining) = response.headers().get_raw("x-ratelimit-remaining") {
			let reqs_remaining = String::from_utf8_lossy(reqs_remaining.one().unwrap())
				.parse::<f32>()
				.unwrap()
				.round() as i32;
			self.remaining.set(Some(reqs_remaining));
		}
		if let Some(secs_remaining) = response.headers().get_raw("x-ratelimit-reset") {
			let secs_remaining = String::from_utf8_lossy(secs_remaining.one().unwrap())
				.parse::<f32>()
				.unwrap()
				.round() as u64;
			self.reset_time.set(
				Instant::now() +
					Duration::new(secs_remaining, 0),
			);
		}

		if !response.status().is_success() {
			return Err(RedditError::BadRequest);
		}

		match json::from_str(&out) {
			Ok(r) => Ok(r),
			Err(_) => Err(RedditError::BadResponse { response: out }),
		}
	}

	/// Send a request to reddit with authorization headers
	pub fn run_auth_request(&self, mut req: Request) -> Result<Value, RedditError> {
		// Check if this connection is authorized
		// This shit's some fuckin spaghetti tho now yo
		// TODO cleanup

		if let Some(ref auth) = self.auth {
			req.headers_mut().set_raw(
				"Authorization",
				format!(
					"Bearer {}",
					match *auth {
						OAuth::Script {
							id: ref _id,
							secret: ref _secret,
							username: ref _username,
							password: ref _password,
							ref token,
						} => token.to_string(),
						OAuth::InstalledApp {
							id: ref _id,
							redirect: ref _redirect,
							ref token,
							ref refresh_token,
							ref expire_instant,
						} => {
							// If the token can expire and we are able to refresh it
							if let (Some(_refresh_token), Some(expire_instant)) = (refresh_token.borrow().clone(), expire_instant.get()) {
								// If the token's expired, refresh it
								if Instant::now() > expire_instant {
									auth.refresh(&self);
								}
								token.borrow().to_string()

							} else if let Some(expire_instant) = expire_instant.get() {
								if Instant::now() > expire_instant {
									return Err(RedditError::Forbidden);
								} else {
									token.borrow().to_string()
								}
							} else {
								token.borrow().to_string()
							}
						}
					}
				),
			);
			self.run_request(req)
		} else {
			Err(RedditError::Forbidden)
		}
	}

	pub fn set_limit(&self, limit: LimitMethod) {
		self.limit.set(limit);
	}
}
