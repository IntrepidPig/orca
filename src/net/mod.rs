/// Contains all functionality for OAuth and logins
pub mod auth;

use std::io::Read;
use std::time::{Duration, Instant};
use std::thread;
use std::cell::Cell;

use json;
use json::Value;
use http::{Client, Request, Method, Url};
use http::header::{UserAgent, Authorization, Bearer};

use errors::*;
use self::auth::{Auth, OauthApp};

pub struct Connection {
	pub auth: Option<auth::Auth>,
	pub useragent: UserAgent,
	pub client: Client,
	lastreq: Cell<Instant>,
}

impl Connection {
	pub fn new(appname: String, appversion: String, appauthor: String) -> Result<Connection> {
		let useragent = UserAgent::new(format!("orca:{}:{} (by {})", appname, appversion, appauthor));
		Ok(Connection { auth: None, useragent, client: Client::new().unwrap(), lastreq: Cell::new(Instant::now()) })
	}
	
	pub fn run_request(&self, req: Request) -> Result<Value> {
		if self.lastreq.get().elapsed() < Duration::new(2, 0) {
			let now = Instant::now();
			let targetinstant = self.lastreq.get() + Duration::new(2, 150000000);
			thread::sleep(targetinstant - now);
		}
		
		let mut response = self.client.execute(req).chain_err(|| "Failed to send request")?;
		let mut out = String::new();
		response.read_to_string(&mut out).chain_err(|| "Nice")?;

		let tmp = Instant::now();
		self.lastreq.set(tmp);

		Ok(json::from_str(&out).chain_err(|| "Couldn't parse json")?)
	}
	
	pub fn run_auth_request(&self, mut req: Request) -> Result<Value> {
		if let Some(ref auth) = self.auth.clone() {
			req.headers_mut().set(Authorization(
				Bearer {
					token: auth.token.clone()
				}
			));
			
			self.run_request(req)
		} else {
			Err(ErrorKind::Unauthorized.into())
		}
	}
}