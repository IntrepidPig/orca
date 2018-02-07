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

use std;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use rand::{self, Rng};

use hyper::{Error as HyperError, Method, Request};
use hyper::server::{Http, NewService, Response, Service};
use hyper::header::{Authorization, Basic};
use futures::Future;
use futures::future::ok;
use futures::sync::oneshot::{self, Sender};
use open;
use url::{self, Url};
use failure::Error;

use errors::RedditError;
use net::Connection;
use net::body_from_map;

/// Contains data for authorization for each OAuth app type
/// Currently only `Script` and `InstalledApp` are supported
#[derive(Clone)]
pub enum OAuthApp {
	/// Where args are (app id, redirect uri)
	InstalledApp {
		/// Id of the app
		id: String,
		/// Redirect url of the installed app
		redirect: String,
		/// Function to generate responses based on the result of the request. The argument is a result
		/// that is Ok with the code recieved if the HTTP callback was successful, and an error enum
		/// if it wasn't. The closure returns a result that is either an Ok(Response) which should be
		/// used most if not all the time, or an Err(Response) to indicate an internal error.
		response_gen: Option<Arc<Fn(Result<String, InstalledAppError>) -> Result<Response, Response>>>,
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

	/// Authorize the app based on input from `OAuthApp` struct.
	/// # Arguments
	/// * `conn` - Connection to authorize with
	/// * `app` - OAuth information to use (`OAuthApp`)
	pub fn new(conn: &Connection, app: &OAuthApp) -> Result<OAuth, Error> {
		// TODO: get rid of unwraps and expects
		use self::OAuthApp::*;
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
				params.insert("username", &username);
				params.insert("password", &password);

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
				ref response_gen,
			} => {
				// Random state string to identify this authorization instance
				let state = rand::thread_rng()
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
					id, state, redirect, scopes
				);

				// Open the auth url in the browser so the user can authenticate the app
				thread::spawn(move || {
					open::that(browser_uri).expect("Failed to open browser");
				});

				// A oneshot future channel that the hyper server has access to to send the code back
				// to this thread.
				let (code_sender, code_reciever) = oneshot::channel::<Result<String, InstalledAppError>>();

				// Convert the redirect url into something parseable by the HTTP server
				let redirect_url = Url::parse(&redirect)?;
				let main_redirect = format!(
					"{}:{}",
					redirect_url.host_str().unwrap_or("127.0.0.1"),
					redirect_url.port().unwrap_or(7878).to_string()
				);

				// Set the default response generator if necessary
				let response_gen = if let &Some(ref response_gen) = response_gen {
					Arc::clone(response_gen)
				} else {
					Arc::new(
						|res: Result<String, InstalledAppError>| -> Result<Response, Response> {
							match res {
								Ok(_) => Ok(Response::new().with_body("Successfully got the code")),
								Err(e) => Err(Response::new().with_body(format!("{}", e))),
							}
						},
					)
				};

				// Create a server with the instance of a NewInstalledAppService struct with the
				// responses given, the oneshot sender and the generated state string
				let mut server = Http::new().bind(
					&main_redirect.as_str().parse()?,
					NewInstalledAppService {
						sender: RefCell::new(Some(code_sender)),
						state: state.clone(),
						response_gen,
					},
				)?;

				// Create a code value that is optional but should be set eventually
				let code: RefCell<Result<String, InstalledAppError>> = RefCell::new(Err(InstalledAppError::NeverRecieved));

				// When the code_reciever oneshot resolves, set the new_code value.
				let finish = code_reciever.then(|new_code| -> Result<(), ()> {
					if let Ok(new_code) = new_code {
						match new_code {
							Ok(new_code) => {
								*code.borrow_mut() = Ok(new_code);
								Ok(())
							}
							Err(e) => {
								*code.borrow_mut() = Err(e);
								Err(())
							}
						}
					} else {
						Err(())
					}
				});

				// Run the server until the code future oneshot resolves and has set the code variable.
				server.run_until(finish)?;

				// Make sure we got the code. Return an error if we didn't.
				let code = match *code.borrow() {
					Ok(ref new_code) => new_code.clone(),
					Err(ref e) => return Err(e.clone().into()),
				};

				// Get the access token with the new code we just got
				let mut params: HashMap<&str, &str> = HashMap::new();
				params.insert("grant_type", "authorization_code");
				params.insert("code", &code);
				params.insert("redirect_uri", &redirect);

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

				if let (Some(expires_in), Some(token), Some(refresh_token), Some(_scope)) = (
					response.get("expires_in"),
					response.get("access_token"),
					response.get("refresh_token"),
					response.get("scope"),
				) {
					Ok(OAuth::InstalledApp {
						id: id.to_string(),
						redirect: redirect.to_string(),
						token: RefCell::new(token.as_str().unwrap().to_string()),
						refresh_token: RefCell::new(Some(refresh_token.to_string())),
						expire_instant: Cell::new(Some(
							Instant::now() + Duration::new(expires_in.to_string().parse::<u64>().unwrap(), 0),
						)),
					})
				} else {
					Err(Error::from(RedditError::AuthError))
				}
			}
		}
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

// The struct that creates new InstalledAppServices when necessary. Basically the same thing as the
// InstalledAppService, not sure why it's even necessary but I'm too scared to touch it now.
struct NewInstalledAppService {
	sender: RefCell<Option<Sender<Result<String, InstalledAppError>>>>,
	state: String,
	response_gen: Arc<Fn(Result<String, InstalledAppError>) -> Result<Response, Response>>,
}

impl NewService for NewInstalledAppService {
	type Request = Request;
	type Response = Response;
	type Error = HyperError;

	type Instance = InstalledAppService;

	fn new_service(&self) -> Result<Self::Instance, std::io::Error> {
		let code_sender = if let Some(sender) = self.sender.pop() {
			println!("Created service with sender");
			RefCell::new(Some(sender))
		} else {
			println!("Didn't have sender for new service");
			RefCell::new(None)
		};

		Ok(InstalledAppService {
			code_sender,
			state: self.state.clone(),
			response_gen: Arc::clone(&self.response_gen),
		})
	}
}

// The service that has the code_sender to send the code back to the main thread, the state to verify
// that this is the right authorization instance, the optional responses, and a tokio Core needed to
// clone the responses.
struct InstalledAppService {
	code_sender: RefCell<Option<Sender<Result<String, InstalledAppError>>>>,
	state: String,
	response_gen: Arc<Fn(Result<String, InstalledAppError>) -> Result<Response, Response>>,
}

impl Service for InstalledAppService {
	type Request = Request;
	type Response = Response;
	type Error = HyperError;

	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Request) -> Self::Future {
		// Get the data from the request (the state and the code, or the error) in a HashMap
		let query_str = req.uri().as_ref();
		let query_str = &query_str[2..query_str.len()];
		let params: HashMap<_, _> = url::form_urlencoded::parse(query_str.as_bytes()).collect();

		fn split(res: Result<Response, Response>) -> Response {
			match res {
				Ok(t) => t,
				Err(t) => {
					error!("User's closure generated an error response {:?}", t);
					t
				}
			}
		}

		fn create_res(gen: &Fn(Result<String, InstalledAppError>) -> Result<Response, Response>, res: Result<String, InstalledAppError>, sender: &RefCell<Option<Sender<Result<String, InstalledAppError>>>>) -> Box<Future<Item = Response, Error = HyperError>> {
			let resp = if let Some(sender) = sender.pop() {
				println!("Got sender");
				let resp = gen(res.clone());
				sender.send(res).unwrap();
				resp
			} else {
				println!("Didn't have sender");
				gen(Err(InstalledAppError::AlreadyRecieved))
			};
			Box::new(ok(split(resp)))
		}

		// If there was an error stop here, returning the error response
		if params.contains_key("error") {
			warn!("Got failed authorization. Error was {}", &params["error"]);
			let err = InstalledAppError::Error {
				msg: params["error"].to_string(),
			};
			create_res(&*self.response_gen, Err(err.clone()), &self.code_sender)
		} else {
			// Get the state if it exists
			let state = if let Some(state) = params.get("state") {
				state
			} else {
				// Return error response if we didn't get the state
				return create_res(
					&*self.response_gen,
					Err(InstalledAppError::MismatchedState),
					&self.code_sender,
				);
			};
			// Error if the state doesn't match
			if *state != self.state {
				error!(
					"State didn't match. Got state \"{}\", needed state \"{}\"",
					state, self.state
				);
				create_res(
					&*self.response_gen,
					Err(InstalledAppError::MismatchedState),
					&self.code_sender,
				)
			} else {
				// Get the code and send it with the oneshot sender back to the main thread
				let code = &params["code"];
				create_res(
					&*self.response_gen,
					Ok(code.clone().into()),
					&self.code_sender,
				)
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
