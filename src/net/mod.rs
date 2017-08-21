/// Contains all functionality for OAuth and logins
pub mod auth;

use std::io::Read;
use std::time::{Duration, Instant};
use std::thread;
use std::cell::Cell;

use json;
use http::{Client, Request, Method, Url};
use http::header::{UserAgent, Authorization, Bearer};

use self::auth::{Auth, OauthApp};

pub struct Connection {
	pub auth: Option<auth::Auth>,
	pub useragent: UserAgent,
	pub client: Client,
	lastreq: Cell<Instant>,
}

impl Connection {
	pub fn new(appname: String, appversion: String, appauthor: String) -> Connection {
		let useragent = UserAgent::new(format!("orca:{}:{} (by {})", appname, appversion, appauthor));
		Connection { auth: None, useragent, client: Client::new().unwrap(), lastreq: Cell::new(Instant::now()) }
	}
	
	pub fn run_request(&self, req: Request) -> Result<json::Value, ()> {
		if self.lastreq.get().elapsed() < Duration::new(2, 0) {
			let now = Instant::now();
			let targetinstant = self.lastreq.get() + Duration::new(2, 150000000);
			thread::sleep(targetinstant - now);
		}
		
		let result = if let Ok(mut response) = self.client.execute(req) {
			let mut out = String::new();
			match response.read_to_string(&mut out) {
				Err(_) => return Err(()),
				_ => {}
			}
			Ok(json::from_str(&out).unwrap())
		} else {
			Err(())
		};
		
		let tmp = Instant::now();
		self.lastreq.set(tmp);
		result
	}
	
	pub fn run_auth_request(&self, mut req: Request) -> Result<json::Value, ()> {
		if let Some(ref auth) = self.auth.clone() {
			req.headers_mut().set(Authorization(
				Bearer {
					token: auth.token.clone()
				}
			));
			
			self.run_request(req)
		} else {
			Err(())
		}
	}
}