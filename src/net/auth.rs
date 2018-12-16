//! # Authorization
//! Authorization for a Reddit client is done by OAuth, which can be done multiple (3) ways. The
//! possible methods of authorization are Script, Installed App, and Web App. Currently, only
//! the first two are supported by orca. There are certain use cases for each app type.
//!
//! ## Scripts
//!
//! Script apps are used when you only want to authorize one user, that you own. This is the app
//! type used for bots. It's special because it can keep a secret (secrets can be stored on the
//! with the client). To create a script app, you first have to register it at
//! [https://www.reddit.com/prefs/apps](https://www.reddit.com/prefs/apps). Make sure you're logged
//! in as the user you want the script to authorize as when you register the app. At the bottom of
//! the page, click "Create New App", and fill in the name, select script type, enter a short
//! description (that will only be seen by you), leave the about url empty, and set the redirect uri
//! to `https://www.example.com`. (We do this because this field is only necessary for installed
//! apps but is required to be filled in anyway.)
//!
//! Once you create the app, a box should pop up that has the name of your app, and then shortly
//! below it a string of random characters. This is the id of the script. Then lower in the
//! properties there should be a field called "secret" with another long string of characters. That
//! is your app's secret.
//!
//! Once you have the id and secret, you can instantiate an `OAuthApp::Script` enum with the id and
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
//! [https://www.reddit.com/prefs/apps](https://www.reddit.com/prefs/apps), and create a new app,
//! this time with the installed type. Fill in the name, set it to installed app, fill in a short
//! description (this time it's visible by anyone using your app), enter an about url if you want,
//! and set the redirect uri to exactly `http://127.0.0.1:7878` (hopefully this will be customizable
//! in the future).
//!
//! When you create this app, the id of the app will be shorly below the name in the box that comes
//! upp. Now in you application code, create an `OAuthApp::InstalledApp` with the id of you app and
//! the redirect uri exactly as you entered it when you registered the app. When you call the
//! `authorize` function with this as a parameter, it will open a web browser with either a reddit
//! login prompt, or if you are already logged in, a request for permission for your app. Once you
//! click allow, the page should redirect to a simple display of the words `Authorization successful`.
//! Hopefully this too will be customizable one day.
//!
//! Installed apps, unlike scripts, require periodic reauthorization, or will expire without the
//! possibility of refreshing if a permanent duration wasn't requested. This should be done
//! automatically by the `net::Connection` instance.

use rand::{self, Rng};
use std;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use base64;
use failure::Error;
use futures::future::ok;
use futures::sync::oneshot::{self, Sender};
use futures::Future;
use hyper::header::{self, HeaderValue};
use hyper::server::Server;
use hyper::service::{MakeService, Service};
use hyper::{Body, Error as HyperError, Method, Request, Response};
use open;
use url::{self, Url};

use errors::RedditError;
use net::body_from_map;
use net::Connection;

/// Function type that is passed into OAuthApp::InstalledApp to generate response from code retrieval.
pub type ResponseGenFn = (Fn(&Result<String, InstalledAppError>) -> Response<Body>) + Send + Sync;

type CodeSender = Arc<Mutex<Option<Sender<Result<String, InstalledAppError>>>>>;

/// Enum representing OAuth information that has been aquired from authorization. This should only be
/// used internally within orca.
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
	pub fn refresh(&self, conn: &Connection) -> Result<(), Error> {
		match *self {
			OAuth::Script { .. } => Ok(()),
			OAuth::InstalledApp {
				ref id,
				redirect: ref _redirect,
				ref token,
				ref refresh_token,
				ref expire_instant,
			} => {
				let old_refresh_token = if let Some(ref refresh_token) = *refresh_token.borrow() { refresh_token.clone() } else { return Err(RedditError::AuthError.into()) };
				// Get the access token with the new code we just got
				let mut params: HashMap<&str, &str> = HashMap::new();
				params.insert("grant_type", "refresh_token");
				params.insert("refresh_token", &old_refresh_token);

				// Request for the access token
				let mut tokenreq = Request::builder().method(Method::POST).uri("https://www.reddit.com/api/v1/access_token/.json").body(body_from_map(&params)).unwrap();
				// httpS is important
				tokenreq.headers_mut().insert(header::AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", { base64::encode(&format!("{}:", id)) })).unwrap());

				// Send the request and get the access token as a response
				let response = conn.run_request(tokenreq)?;

				if let (Some(expires_in), Some(new_token), Some(scope)) = (response.get("expires_in"), response.get("access_token"), response.get("scope")) {
					let expires_in = expires_in.as_u64().unwrap();
					let new_token = new_token.as_str().unwrap();
					let _scope = scope.as_str().unwrap();
					*token.borrow_mut() = new_token.to_string();
					expire_instant.set(Some(Instant::now() + Duration::new(expires_in.to_string().parse::<u64>().unwrap(), 0)));

					Ok(())
				} else {
					Err(Error::from(RedditError::AuthError))
				}
			}
		}
	}

	/// Authorize the app as a script
	/// # Arguments
	/// * `conn` - A refernce to the connection to authorize
	/// * `id` - The app id registered on Reddit
	/// * `secret` - The app secret registered on Reddit
	/// * `username` - The username of the user to authorize as
	/// * `password` - The password of the user to authorize as
	pub fn create_script(conn: &Connection, id: &str, secret: &str, username: &str, password: &str) -> Result<OAuth, Error> {
		// authorization paramaters to request
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("grant_type", "password");
		params.insert("username", &username);
		params.insert("password", &password);

		// Request for the bearer token
		let mut tokenreq = Request::builder().method(Method::POST).uri("https://ssl.reddit.com/api/v1/access_token/.json").body(body_from_map(&params)).unwrap();
		// httpS is important
		tokenreq.headers_mut().insert(header::AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", { base64::encode(&format!("{}:{}", id, secret)) })).unwrap());

		// Send the request and get the bearer token as a response
		let response = conn.run_request(tokenreq)?;

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
			Err(RedditError::AuthError.into())
		}
	}

	/// Authorize the app as an installed app
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
	pub fn create_installed_app<I: Into<Option<Arc<ResponseGenFn>>>>(conn: &Connection, id: &str, redirect: &str, response_gen: I, scopes: &Scopes) -> Result<OAuth, Error> {
		let response_gen = response_gen.into();
		// Random state string to identify this authorization instance
		let state = rand::thread_rng().gen_ascii_chars().take(16).collect::<String>();

		let scopes = &scopes.to_string();
		let browser_uri = format!(
			"https://www.reddit.com/api/v1/authorize?client_id={}&response_type=code&\
			 state={}&redirect_uri={}&duration=permanent&scope={}",
			id, state, redirect, scopes
		);

		let state_rc = Arc::new(state);

		// Open the auth url in the browser so the user can authenticate the app
		thread::spawn(move || {
			open::that(browser_uri).expect("Failed to open browser");
		});

		// A oneshot future channel that the hyper server has access to to send the code back
		// to this thread.
		let (code_sender, code_reciever) = oneshot::channel::<Result<String, InstalledAppError>>();

		// Convert the redirect url into something parseable by the HTTP server
		let redirect_url = Url::parse(&redirect)?;
		let main_redirect = format!("{}:{}", redirect_url.host_str().unwrap_or("127.0.0.1"), redirect_url.port().unwrap_or(7878).to_string());

		// Set the default response generator if necessary
		let response_gen = if let Some(ref response_gen) = response_gen {
			Arc::clone(response_gen)
		} else {
			Arc::new(|res: &Result<String, InstalledAppError>| -> Response<Body> {
				match res {
					Ok(_) => Response::new("Successfully got the code".into()),
					Err(e) => Response::new(format!("{}", e).into()),
				}
			})
		};

		// Create a server with the instance of a NewInstalledAppService struct with the
		// responses given, the oneshot sender and the generated state string
		let server = Server::bind(&main_redirect.as_str().parse()?).serve(MakeInstalledAppService {
			code_sender: Arc::new(Mutex::new(Some(code_sender))),
			state: Arc::clone(&state_rc),
			response_gen: Arc::clone(&response_gen),
		});

		// Create a code value that is optional but should be set eventually
		let code: Arc<Mutex<Result<String, InstalledAppError>>> = Arc::new(Mutex::new(Err(InstalledAppError::NeverRecieved)));
		let code_clone = Arc::clone(&code);

		// When the code_reciever oneshot resolves, set the new_code value.
		let finish = code_reciever.then(move |new_code| {
			let code = code_clone;
			if let Ok(new_code) = new_code {
				match new_code {
					Ok(new_code) => {
						*code.lock().unwrap() = Ok(new_code);
						Ok(())
					}
					Err(e) => {
						*code.lock().unwrap() = Err(e);
						Err(())
					}
				}
			} else {
				Err(())
			}
		});

		let graceful = server.with_graceful_shutdown(finish).map_err(|e| eprintln!("Server failed: {}", e));

		// Run the server until the code future oneshot resolves and has set the code variable.
		hyper::rt::run(graceful);

		// Make sure we got the code. Return an error if we didn't.
		let code = match *code.lock().unwrap() {
			Ok(ref new_code) => new_code.clone(),
			Err(ref e) => return Err(e.clone().into()),
		};

		// Get the access token with the new code we just got
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("grant_type", "authorization_code");
		params.insert("code", &code);
		params.insert("redirect_uri", &redirect);

		// Request for the access token
		let mut tokenreq = Request::builder().method(Method::POST).uri("https://ssl.reddit.com/api/v1/access_token/.json").body(body_from_map(&params)).unwrap();
		// httpS is important
		tokenreq.headers_mut().insert(header::AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", base64::encode(&format!("{}:", id)))).unwrap());

		// Send the request and get the access token as a response
		let response = conn.run_request(tokenreq)?;

		if let (Some(expires_in), Some(token), Some(refresh_token), Some(scope)) = (response.get("expires_in"), response.get("access_token"), response.get("refresh_token"), response.get("scope")) {
			let expires_in = expires_in.as_u64().unwrap();
			let token = token.as_str().unwrap();
			let refresh_token = refresh_token.as_str().unwrap();
			let _scope = scope.as_str().unwrap();
			Ok(OAuth::InstalledApp {
				id: id.to_string(),
				redirect: redirect.to_string(),
				token: RefCell::new(token.to_string()),
				refresh_token: RefCell::new(Some(refresh_token.to_string())),
				expire_instant: Cell::new(Some(Instant::now() + Duration::new(expires_in.to_string().parse::<u64>().unwrap(), 0))),
			})
		} else {
			Err(Error::from(RedditError::AuthError))
		}
	}
}

/// A struct representing scopes that an installed app can request permission for.
/// To use, create an instance of the struct and set the fields you want to use to true.
///
/// Note: In the field documentation, "the user" refers to the currently authorized user
pub struct Scopes {
	/// See detailed info about the user
	pub identity: bool,
	/// Edit posts of the user
	pub edit: bool,
	/// Flair posts of the user
	pub flair: bool,
	/// Unknown
	pub history: bool,
	/// Unknown
	pub modconfig: bool,
	/// Unknown
	pub modflair: bool,
	/// Unknown
	pub modlog: bool,
	/// Unknown
	pub modposts: bool,
	/// Unknown
	pub modwiki: bool,
	/// Unknown
	pub mysubreddits: bool,
	/// Unknown
	pub privatemessages: bool,
	/// Unknown
	pub read: bool,
	/// Report posts on behalf of the user
	pub report: bool,
	/// Save posts to the user's account
	pub save: bool,
	/// Submit posts on behalf of the user
	pub submit: bool,
	/// Unknown
	pub subscribe: bool,
	/// Vote on things on behalf of the user
	pub vote: bool,
	/// Unknown
	pub wikiedit: bool,
	/// Unknown
	pub wikiread: bool,
	/// Unknown
	pub account: bool,
}

impl Scopes {
	/// Create a scopes instance with no permissions requested
	pub fn empty() -> Scopes {
		Scopes {
			identity: false,
			edit: false,
			flair: false,
			history: false,
			modconfig: false,
			modflair: false,
			modlog: false,
			modposts: false,
			modwiki: false,
			mysubreddits: false,
			privatemessages: false,
			read: false,
			report: false,
			save: false,
			submit: false,
			subscribe: false,
			vote: false,
			wikiedit: false,
			wikiread: false,
			account: false,
		}
	}

	/// Create a scopes instance with all permissions requested
	pub fn all() -> Scopes {
		Scopes {
			identity: true,
			edit: true,
			flair: true,
			history: true,
			modconfig: true,
			modflair: true,
			modlog: true,
			modposts: true,
			modwiki: true,
			mysubreddits: true,
			privatemessages: true,
			read: true,
			report: true,
			save: true,
			submit: true,
			subscribe: true,
			vote: true,
			wikiedit: true,
			wikiread: true,
			account: true,
		}
	}

	/// Convert the struct to a string representation to be sent to Reddit
	fn to_string(&self) -> String {
		let mut string = String::new();
		if self.identity {
			string.push_str("identity");
		}
		if self.edit {
			string.push_str(",edit");
		}
		if self.flair {
			string.push_str(",flair");
		}
		if self.history {
			string.push_str(",history");
		}
		if self.modconfig {
			string.push_str(",modconfig");
		}
		if self.modflair {
			string.push_str(",modflair");
		}
		if self.modlog {
			string.push_str(",modlog");
		}
		if self.modposts {
			string.push_str(",modposts");
		}
		if self.modwiki {
			string.push_str(",modwiki");
		}
		if self.mysubreddits {
			string.push_str(",mysubreddits");
		}
		if self.privatemessages {
			string.push_str(",privatemessages");
		}
		if self.read {
			string.push_str(",read");
		}
		if self.report {
			string.push_str(",report");
		}
		if self.save {
			string.push_str(",save");
		}
		if self.submit {
			string.push_str(",submit");
		}
		if self.subscribe {
			string.push_str(",subscribe");
		}
		if self.vote {
			string.push_str(",vote");
		}
		if self.wikiedit {
			string.push_str(",wikiedit");
		}
		if self.wikiread {
			string.push_str(",wikiread");
		}
		if self.account {
			string.push_str(",account");
		}

		string
	}
}

/// Enum that contains possible errors from a request for the OAuth Installed App type.
#[derive(Debug, Fail, Clone)]
pub enum InstalledAppError {
	/// Got a generic error in the request
	#[fail(display = "Got an unknown error: {}", msg)]
	Error {
		/// The message included in the error
		msg: String,
	},
	/// The state string wasn't present or did not match
	#[fail(display = "The states did not match")]
	MismatchedState,
	/// The code has already been recieved
	#[fail(display = "A code was already recieved")]
	AlreadyRecieved,
	/// No message was ever recieved
	#[fail(display = "No message was ever recieved")]
	NeverRecieved,
}

struct MakeInstalledAppService {
	code_sender: CodeSender,
	state: Arc<String>,
	response_gen: Arc<ResponseGenFn>,
}

impl<Ctx> MakeService<Ctx> for MakeInstalledAppService {
	type ReqBody = Body;
	type ResBody = Body;
	type Error = hyper::Error;
	type Service = InstalledAppService;
	type Future = Box<Future<Item = Self::Service, Error = Self::MakeError> + Send + Sync>;
	type MakeError = Box<dyn std::error::Error + Send + Sync>;

	fn make_service(&mut self, _ctx: Ctx) -> Self::Future {
		Box::new(futures::future::ok(InstalledAppService {
			code_sender: Arc::clone(&self.code_sender),
			state: Arc::clone(&self.state),
			response_gen: Arc::clone(&self.response_gen),
		}))
	}
}

// The service that has the code_sender to send the code back to the main thread, the state to verify
// that this is the right authorization instance, the optional responses, and a tokio Core needed to
// clone the responses.
struct InstalledAppService {
	code_sender: CodeSender,
	state: Arc<String>,
	response_gen: Arc<ResponseGenFn>,
}

impl Service for InstalledAppService {
	type ReqBody = Body;
	type ResBody = Body;
	type Error = HyperError;
	type Future = Box<Future<Item = Response<Self::ResBody>, Error = Self::Error> + Send>;

	fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
		// Get the data from the request (the state and the code, or the error) in a HashMap
		let query_str = req.uri().path_and_query().unwrap().as_str();
		let query_str = &query_str[2..query_str.len()];
		let params: HashMap<_, _> = url::form_urlencoded::parse(query_str.as_bytes()).collect();

		// Create a HTTP response based on the result of the code retrieval, the code sender, and the
		// response generator.
		fn create_res(gen: &ResponseGenFn, res: &Result<String, InstalledAppError>, sender: &CodeSender) -> <InstalledAppService as Service>::Future {
			let mut sender = sender.lock().unwrap();
			let sender = if let Some(sender) = sender.take() {
				sender
			} else {
				return Box::new(ok(gen(&Err(InstalledAppError::AlreadyRecieved))));
			};
			let resp = match sender.send(res.clone()) {
				Ok(_) => gen(&res),
				Err(_) => gen(&Err(InstalledAppError::AlreadyRecieved)),
			};
			Box::new(ok(resp))
		}

		// If there was an error stop here, returning the error response
		if params.contains_key("error") {
			warn!("Got failed authorization. Error was {}", &params["error"]);
			let err = InstalledAppError::Error { msg: params["error"].to_string() };
			create_res(&*self.response_gen, &Err(err.clone()), &self.code_sender)
		} else {
			// Get the state if it exists
			let state = if let Some(state) = params.get("state") {
				state
			} else {
				// Return error response if we didn't get the state
				return create_res(&*self.response_gen, &Err(InstalledAppError::MismatchedState), &self.code_sender);
			};
			// Error if the state doesn't match
			if *state != *self.state {
				error!("State didn't match. Got state \"{}\", needed state \"{}\"", state, self.state);
				create_res(&*self.response_gen, &Err(InstalledAppError::MismatchedState), &self.code_sender)
			} else {
				// Get the code and send it with the oneshot sender back to the main thread
				let code = &params["code"];
				create_res(&*self.response_gen, &Ok(code.clone().into()), &self.code_sender)
			}
		}
	}
}

// A neat trait I came up with. If you have a RefCell<Option<T>>, then you can call pop() on it and
// it will take the value out of the RefCell and give it back. If it doesn't exist, then it just returns None.
trait RefCellExt<T> {
	fn pop(&self) -> Option<T>;
}

impl<T: std::fmt::Debug> RefCellExt<T> for RefCell<Option<T>> {
	fn pop(&self) -> Option<T> {
		if self.borrow().is_some() {
			return std::mem::replace(&mut *self.borrow_mut(), None);
		}

		None
	}
}
