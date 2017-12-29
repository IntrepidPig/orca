use std::collections::HashMap;
use std::thread;
use std::time::{Instant, Duration};
use std::cell::{Cell, RefCell};
use rand::{self, Rng};

use hyper::{Request, Method};
use hyper::header::{Authorization, Basic};
use open;
use tiny_http::{Server, Response};
use url;

use net::Connection;

use net::body_from_map;
use net::error::AuthError;
use failure::Error;

/// Contains data for authorization for each OAuth app type
/// Currently only Script and InstalledApp are supported
#[derive(Debug, Clone)]
pub enum OauthApp {
	/// Not Implemented
	WebApp,
	/// Where args are (app id, redirect uri)
	InstalledApp { id: String, redirect: String },
	/// Where args are (app id, app secret, username, password)
	Script {
		id: String,
		secret: String,
		username: String,
		password: String,
	},
}

#[derive(Debug, Clone)]
pub enum OAuth {
	Script {
		id: String,
		secret: String,
		username: String,
		password: String,
		token: String,
	},
	InstalledApp {
		id: String,
		redirect: String,
		token: RefCell<String>,
		refresh_token: RefCell<Option<String>>,
		expire_instant: Cell<Option<Instant>>,
	},
}

impl OAuth {
	pub fn refresh(&self, _conn: &Connection) {
		unimplemented!();
	}

	pub fn new(conn: &Connection, app: &OauthApp) -> Result<OAuth, Error> {
		// TODO: get rid of unwraps and expects
		use self::OauthApp::*;
		match *app {
			Script {
				ref id,
				ref secret,
				ref username,
				ref password,
			} => {
				// authorization paramaters to request
				let mut params: HashMap<&str, &str> = HashMap::new();
				params.insert("grant_type", "password");
				params.insert("username", username);
				params.insert("password", password);

				// Request for the bearer token
				let mut tokenreq = Request::new(
					Method::Post,
					"https://ssl.reddit.com/api/v1/access_token/.json".parse()?,
				); // httpS is important
				tokenreq.set_body(body_from_map(&params));
				tokenreq.headers_mut().set(Authorization(Basic {
					username: id.clone(),
					password: Some(secret.clone()),
				}));

				// Send the request and get the bearer token as a response
				let mut response = conn.run_request(tokenreq)?;

				if let Some(token) = response.get("access_token") {
					let token = token.as_str().unwrap().to_string();
					Ok(OAuth::Script {
						id: id.to_string(),
						secret: secret.to_string(),
						username: username.to_string(),
						password: password.to_string(),
						token,
					})
				} else {
					Err(Error::from(AuthError {}))
				}
			}
			InstalledApp {
				ref id,
				ref redirect,
			} => {
				// Random state string to identify this authorization instance
				let state = &rand::thread_rng()
					.gen_ascii_chars()
					.take(16)
					.collect::<String>();

				// Permissions (scopes) to authorize, should be customizable in the future
				let scopes = "identity,edit,flair,history,modconfig,modflair,modlog,modposts,\
				                     modwiki,mysubreddits,privatemessages,read,report,save,submit,\
				                     subscribe,vote,wikiedit,wikiread,account"; // TODO customizable

				let browser_uri = format!(
					"https://www.reddit.com/api/v1/authorize?client_id={}&response_type=code&\
				            state={}&redirect_uri={}&duration=permanent&scope={}",
					id,
					state,
					redirect,
					scopes
				);

				// Open the auth url in the browser so the user can authenticate the app
				thread::spawn(move || {
					open::that(browser_uri).expect("Failed to open browser");
				});

				// Start http server to recieve the request from the redirect uri
				let server = Server::http("127.0.0.1:7878").unwrap();

				// Parse request url to get token and stuff
				let req = server.recv().unwrap();
				let params = {
					let iter = url::form_urlencoded::parse(&req.url()[2..].as_bytes()).into_owned(); // Substring to cut out "/?"
					let mut map: HashMap<String, String> = HashMap::new();

					for i in iter {
						map.insert(i.0, i.1);
					}
					map
				};
				req.respond(Response::from_string("Authorization successful"))
					.unwrap(); // TODO make this customizable

				if let (Some(new_state), Some(code)) = (params.get("state"), params.get("code")) {
					if new_state != state {
						return Err(Error::from(AuthError {}));
					}

					let mut params: HashMap<&str, &str> = HashMap::new();
					params.insert("grant_type", "authorization_code");
					params.insert("code", code);
					params.insert("redirect_uri", redirect);

					// Request for the access token
					let mut tokenreq = Request::new(
						Method::Post,
						"https://ssl.reddit.com/api/v1/access_token/.json".parse()?,
					); // httpS is important
					tokenreq.set_body(body_from_map(&params));
					tokenreq.headers_mut().set(Authorization(Basic {
						username: id.clone(),
						password: None,
					}));

					// Send the request and get the access token as a response
					let mut response = conn.run_request(tokenreq)?;

					if let (Some(expires_in), Some(token), Some(refresh_token), Some(_scope)) =
						(
							response.get("expires_in"),
							response.get("access_token"),
							response.get("refresh_token"),
							response.get("scope"),
						)
					{
						Ok(OAuth::InstalledApp {
							id: id.to_string(),
							redirect: redirect.to_string(),
							token: RefCell::new(token.as_str().unwrap().to_string()),
							refresh_token: RefCell::new(Some(refresh_token.to_string())),
							expire_instant: Cell::new(Some(
								Instant::now() +
									Duration::new(
										expires_in.to_string().parse::<u64>().unwrap(),
										0,
									),
							)),
						})
					} else {
						Err(Error::from(AuthError {}))
					}
				} else {
					Err(Error::from(AuthError {}))
				}
			}
			// App types other than script and installed are unsupported right now
			_ => unimplemented!(),
		}
	}
}
