//! # Authorization
//! Authorization for a Reddit client is done by OAuth, which can be done multiple (3) ways. The
//! possible methods of authorization are Script, Installed App, and Web App. Currently, only
//! the first two are supported by orca. There are certain use cases for each app type.
//!
//! ## Scripts
//!
//! Script apps are used when you only want to authorize one user, that you own. This is the app
//! type used for bots. It's special because it can keep a secret (secrets can be stored on the
//! with the client). To create a script app, you first have to register it at https://www.reddit.com/prefs/apps.
//! Make sure you're logged in as the user you want the script to authorize as when you register the
//! app. At the bottom of the page, click "Create New App", and fill in the name, select script type, enter
//! a short description (that will only be seen by you), leave the about url empty, and set the
//! redirect uri to `https://www.example.com`. (We do this because this field is only necessary for
//! installed apps but is required to be filled in anyway.)
//!
//! Once you create the app, a box should pop up that has the name of your app, and then shortly
//! below it a string of random characters. This is the id of the script. Then lower in the
//! properties there should be a field called "secret" with another long string of characters. That
//! is your app's secret.
//!
//! Once you have the id and secret, you can instantiate an `OauthApp::Script` enum with the id and
//! secret of the script and the username and password of the user that registered the app, and
//! pass it into the `authorize` function of an `App` instance.
//!
//! ## Installed Apps
//!
//! Installed apps are used when you want your program to be able to be authorized as any user that
//! is using it. They are unable to keep a secret, so it is more complicated to authorize them.
//! An installed app has no secret id. Instead, it requires that the user visits a url to reddit.com
//! containing info for authorization. After authorizing, reddit.com will redirect the web browser
//! to the redirect uri specified during the app registration, with the tokens requested as
//! parameters. The redirect uri is usually the loopback address with a custom port, and the app
//! starts an HTTP server to recieve that request and the tokens included.
//!
//! Most of this work is implemented for you by orca. At the moment, there is some lacking in
//! customizability, but that will hopefully change in the future. Currently, orca opens the
//! reddit.com in the default browser using the `open` crate, and the redirect uri must always be
//! 127.0.0.1:7878.
//!
//! To create an installed app, the process at first is similar to Script app types. Visit
//! https://www.reddit.com/prefs/apps, and create a new app, this time with the installed type. Fill
//! in the name, set it to installed app, fill in a short description (this time it's visible by
//! anyone using your app), enter an about url if you want, and set the redirect uri to exactly
//! `http://127.0.0.1:7878` (hopefully this will be customizable in the future).
//!
//! When you create this app, the id of the app will be shorly below the name in the box that comes
//! upp. Now in you application code, create an `OauthApp::InstalledApp` with the id of you app and
//! the redirect uri exactly as you entered it when you registered the app. When you call the
//! `authorize` function with this as a parameter, it will open a web browser with either a reddit
//! login prompt, or if you are already logged in, a request for permission for your app. Once you
//! click allow, the page should redirect to a simple display of the words `Authorization successful`.
//! Hopefully this too will be customizable one day.
//!
//! Installed apps, unlike scripts, require periodic reauthorization, or will expire without the
//! possibility of refreshing if a permanent duration wasn't requested. This should be done
//! automatically by the `net::Connection` instance. (Currently not implemented, sorry).

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
use failure::Error;

use errors::RedditError;
use net::Connection;
use net::body_from_map;


/// Contains data for authorization for each OAuth app type
/// Currently only Script and InstalledApp are supported
#[derive(Debug, Clone)]
pub enum OauthApp {
	/// Not Implemented
	WebApp,
	/// Where args are (app id, redirect uri)
	InstalledApp {
		/// Id of the app
		id: String,
		/// Redirect url of the installed app
		redirect: String
	},
	/// Where args are (app id, app secret, username, password)
	Script {
		/// Id of the script
		id: String,
		/// Secret of the script
		secret: String,
		/// Username of the user that owns the script
		username: String,
		/// Password of the user that owns the script
		password: String,
	},
}

/// Enum representing OAuth information that has been aquired from authorization
#[derive(Debug, Clone)]
pub enum OAuth {
	/// Script app type
	Script {
		/// Id of the script
		id: String,
		/// Secret of the script
		secret: String,
		/// Username of the script user
		username: String,
		/// Password of the script user
		password: String,
		/// Token retrieved from script authorization
		token: String,
	},
	/// Installed app type
	InstalledApp {
		/// Id of the installed app
		id: String,
		/// Redirect url of the installed app
		redirect: String,
		/// Token currently in use
		token: RefCell<String>,
		/// The refresh token (to be used to retrieve a new token once the current one expires).
		/// Not present if temporary authorization was requested
		refresh_token: RefCell<Option<String>>,
		/// Instant when the current token expires
		expire_instant: Cell<Option<Instant>>,
	},
}

impl OAuth {
	/// Refreshes the token (only necessary for installed app types)
	pub fn refresh(&self, _conn: &Connection) {
		unimplemented!();
	}

	/// Authorize the app based on input from `OauthApp` struct.
	/// # Arguments
	/// * `conn` - Connection to authorize with
	/// * `app` - OAuth information to use (`OauthApp`)
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
					Err(Error::from(RedditError::AuthError))
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
						return Err(Error::from(RedditError::AuthError));
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
						Err(Error::from(RedditError::AuthError))
					}
				} else {
					Err(Error::from(RedditError::AuthError))
				}
			}
			// App types other than script and installed are unsupported right now
			_ => unimplemented!(),
		}
	}
}
