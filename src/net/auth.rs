//! # Authorization
//! 
//! Orca currently allows API clients to authorize with Reddit in two ways: as a script, and as
//! an installed app. For more info on these app types, see 
//! [Reddit's documentation on them](https://github.com/reddit-archive/reddit/wiki/OAuth2-App-Types).

use std::{
	sync::{Arc, Mutex},
	time::{Instant, Duration},
	collections::{HashMap},
	fmt,
};

use hyper::{
	header::{self, HeaderValue},
	server::{Server},
	service::{self},
	Request, Response, Body, Method,
};
use url::{
	form_urlencoded,
};
use snafu::Snafu;
use futures::{
	channel::{
		oneshot::{self},
	},
};

use crate::{
	Reddit, RedditError,
};

/// Holds info about the current authorization state of the Reddit instance
#[derive(Debug, Clone)]
pub enum OAuth {
	/// Info about a script app type
	Script(ScriptOAuth),
	/// Info about an installed app type
	InstalledApp(InstalledAppOAuth),
}

/// Info about a script app's authorization state
#[derive(Debug, Clone)]
pub struct ScriptOAuth {
	/// The method used for authorizing, useful for re-authorization
	pub method: ScriptAuthMethod,
	/// The current bearer token to be attached to requests to authorize
	pub token: String,
}

/// Info about an installed app's authorization state
#[derive(Debug, Clone)]
pub struct InstalledAppOAuth {
	/// The id of the app as given by Reddit
	pub id: String,
	/// The redirect URL of the app exactly as it appears in Reddit
	pub redirect: String,
	/// The current bearer token to be attached to requests to authorize
	pub token: String,
	/// The token necessary to refresh the current access token
	pub refresh_token: String,
	/// The instant at which the current access token will be expired
	pub expire_instant: Instant,
}

/// Holds info about the current method of attempting a first authorization for a Reddit instance
#[derive(Debug, Clone)]
pub enum AuthMethod {
	/// Info about authorization as a script app
	Script(ScriptAuthMethod),
	/// Info about authorization as an installed app
	InstalledApp(InstalledAppAuthMethod),
}

/// Info about authorization as a script app
#[derive(Debug, Clone)]
pub struct ScriptAuthMethod {
	/// The id of the app as given by Reddit
	pub id: String,
	/// The secret of the app as given by Reddit
	pub secret: String,
	/// The username of the account to login as
	pub username: String,
	/// The password of the account to login as
	pub password: String,
}

/// Info about authorization as an installed app
#[derive(Clone)]
pub struct InstalledAppAuthMethod {
	/// The id of the app as given by Reddit
	pub id: String,
	/// The redirect URL of the app exactly as it appears in Reddit
	pub redirect: String,
	/// Optional function to use to generate HTTP responses to requests to the redirect URL. If `None` is passed,
	/// very basic defaults will be chosen. 
	pub response_gen: Option<Arc<dyn Fn(&Result<(), InstalledAppError>) -> Response<Body> + Send + Sync + 'static>>,
	/// The scopes the app is requesting permission for
	pub scopes: Scopes,
}

impl fmt::Debug for InstalledAppAuthMethod {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("InstalledAppAuthMethod")
			.field("id", &self.id)
			.field("redirect", &self.redirect)
			.field("response_gen", self.response_gen.as_ref().map(|_| &"Some(_)").unwrap_or(&"None"))
			.field("scopes", &self.scopes)
			.finish()
	}
}

macro_rules! define_scopes {
	($($scope:ident),* $(,)?) => {
		#[derive(Debug, Copy, Clone, PartialEq, Eq)]
		/// All scopes possible
		pub struct Scopes {
			$(
				///
				pub $scope: bool,
			)*
		}
		
		impl Scopes {
			/// No scopes chosen
			pub fn empty() -> Self {
				Self {
					$(
						$scope: false,
					)*
				}
			}
			
			/// All scopes chosen
			pub fn all() -> Self {
				Self {
					$(
						$scope: true,
					)*
				}
			}
			
			/// Create a new [`Scopes`](Scopes) object that includes only the scopes `self` and
			/// `other` both have enabled.
			pub fn and(self, other: Self) -> Self {
				Self {
					$(
						$scope: self.$scope && other.$scope,
					)*
				}
			}
			
			/// Create a new [`Scopes`](Scopes) object that includes the scopes that either `self`
			/// or `other` have enabled.
			pub fn or(self, other: Self) -> Self {
				Self {
					$(
						$scope: self.$scope || other.$scope,
					)*
				}
			}
			
			/// Convert this Scopes object to its comma-separated string representation as expected by Reddit
			pub fn to_string(self) -> String {
				let mut buf = String::new();
				$(
					if self.$scope {
						buf.push_str(stringify!($scope));
						buf.push_str(",");
					}
				)*
				if !buf.is_empty() {
					// Remove trailing comma
					buf.pop().unwrap();
				}
				buf
			}
		}
	}
}

define_scopes!(
	identity,
	edit,
	flair,
	history,
	modconfig,
	modflair,
	modlog,
	modposts,
	modwiki,
	mysubreddits,
	privatemessages,
	read,
	report,
	save,
	submit,
	subscribe,
	vote,
	wikiedit,
	wikiread,
	account,
);

impl Reddit {
	/// Try to authorize this Reddit instance with the given method
	pub async fn authorize(&self, method: AuthMethod) -> Result<(), RedditError> {
		match method {
			AuthMethod::Script(script) => {
				let ScriptAuthMethod { id, secret, username, password } = script;
				self.authorize_script(id, secret, username, password).await
			},
			AuthMethod::InstalledApp(installed) => {
				let InstalledAppAuthMethod { id, redirect, response_gen, scopes } = installed;
				self.authorize_installed_app(id, redirect, response_gen, scopes).await
			}
		}
	}
	
	/// Try to authorize this Reddit instance as a script
	/// 
	/// ## Parameters
	/// - `id`: The id of this script as given by Reddit
	/// - `secret`: The secret of this script as given by Reddit
	/// - `username`: The username of the account to login as
	/// - `password`: The password of the account to login as
	pub async fn authorize_script(&self, id: String, secret: String, username: String, password: String) -> Result<(), RedditError> {
		let mut params = form_urlencoded::Serializer::new(String::new());
		params.append_pair("grant_type", "password");
		params.append_pair("username", &username);
		params.append_pair("password", &password);
		let params = params.finish();
		let body = Body::from(params);
		
		let mut token_req = Request::builder()
			.uri("https://ssl.reddit.com/api/v1/access_token/.json")
			.method(Method::POST)
			.header(
				header::AUTHORIZATION,
				HeaderValue::from_str(&format!("Basic {}", base64::encode(&format!("{}:{}", id, secret))))
						.map_err(|_e| {
							log::error!("Failed to create Authorization header");
							RedditError::Unknown
						})?
			)
			.body(body)
			.map_err(|_e| {
				log::error!("Failed to create token request");
				RedditError::Unknown
			})?;
		self.add_user_agent_header(&mut token_req)?;
		
		let response_json: json::Value = self.json_raw_request(token_req).await?;
		if let Some(token) = response_json.get("access_token") {
			let token = token.as_str().unwrap().to_owned();
			*self.auth.write().unwrap() = Some(OAuth::Script(ScriptOAuth {
				method: ScriptAuthMethod {
					id,
					secret,
					username,
					password,
				},
				token,
			}));
			Ok(())
		} else {
			Err(RedditError::Unknown)
		}
	}
	
	/// Tries to authorize the current Reddit instance as an installed app.
	/// 
	/// ## Parameters
	/// - `id`: The id of the app as given by Reddit
	/// - `redirect`: The redirect URL exactly as specified on Reddit
	/// - `response_gen`: Optional function to use to generate HTTP responses to requests to the redirect URL. If `None` is passed,
	/// very basic defaults will be chosen.
	/// - `scopes`: The scopes the app is requesting permission for
	pub async fn authorize_installed_app(
		&self,
		id: String,
		redirect: String,
		response_gen: Option<Arc<dyn Fn(&Result<(), InstalledAppError>) -> Response<Body> + Send + Sync + 'static>>,
		scopes: Scopes,
	) -> Result<(), RedditError> {
		use rand::Rng;
		
		let state = (0..16).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric)).collect::<String>();		
		let scopes_str = scopes.to_string();
		let mut params = form_urlencoded::Serializer::new(String::new());
		params.append_pair("client_id", &id);
		params.append_pair("response_type", "code");
		params.append_pair("state", &state);
		params.append_pair("redirect_uri", &redirect);
		params.append_pair("duration", "permanent"); // TODO allow temporary
		params.append_pair("scope", &scopes_str);
		let params = params.finish();
		let browser_uri = format!("https://www.reddit.com/api/v1/authorize?{}", params);
		let state = Arc::new(state);
		
		// This process should probably be customizable by the API client, like `response_gen`.
		std::thread::spawn(move || open::that(browser_uri).expect("Failed to open browser"));
		
		// Create oneshot futures channels representing the code retrieval process and the shutdown signal for the server.
		// The senders are wrapped in Arc<Mutex<Option<T>>> so that each hyper service can have access to them, and the
		// first service to recieve the code will take ownership of and signal each of them.
		let (code_tx, code_rx) = oneshot::channel::<String>();
		let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
		let code_tx = Arc::new(Mutex::new(Some(code_tx)));
		let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));
		
		// Set the default response generator to a very basic one.
		let response_gen = response_gen
			.unwrap_or_else(|| {
				Arc::new(|res: &Result<(), InstalledAppError>| -> Response<Body> {
					match res {
						Ok(_) => Response::new(Body::from("Successfully authorized")),
						Err(e) => Response::new(format!("Failed to authorize app: {}", e).into()), // TODO
					}
				})
			});
		
		// Format the redirect URL in a way that `Server::bind` understands.
		let redirect_url = url::Url::parse(&redirect)
			.map_err(|_| RedditError::Unknown)?;
		let main_redirect = format!("{}:{}", redirect_url.host_str().unwrap_or("127.0.0.1"), redirect_url.port().unwrap_or(7878));
		
		// Start the server that listens on the redirect URL for redirects from Reddit that will hopefully contain the code, but may
		// contain the message that the user declined to authorize the app or some other error occurred.
		let server
			= Server::bind(&main_redirect.as_str().parse().map_err(|_| RedditError::Unknown)?)
			.serve(service::make_service_fn(|_target| {
				// Clone all the necessary Arcs for each service
				let response_gen = Arc::clone(&response_gen);
				let code_tx = Arc::clone(&code_tx);
				let shutdown_tx = Arc::clone(&shutdown_tx);
				let state = Arc::clone(&state);
				async move {
					Ok::<_, Box<dyn std::error::Error + Send + Sync>>(service::service_fn(move |req: Request<Body>| {
						// Clone all the necessary Arcs for each service (again, tbh I don't understand how this works)
						let response_gen = Arc::clone(&response_gen);
						let shutdown_tx = Arc::clone(&shutdown_tx);
						let code_tx = code_tx.clone();
						let state = Arc::clone(&state);
						async move {
							// Parse the parameters of the redirect request
							let params = form_urlencoded::parse(req.uri().query().unwrap().as_bytes()).collect::<HashMap<_, _>>();
							
							// Create a result that represents the state of the authorization based on this redirect.
							let result = if params.contains_key("error") {
								// This is most likely because the user declined to authorize the app. Too many scopes requested?
								log::warn!("Got failed authorization: Error was: '{}'", params["error"]);
								Err(InstalledAppError::Error {
									msg: params["error"].as_ref().to_owned(),
								})
							} else {
								let received_state = &params["state"];
								if *received_state != *state {
									// The states didn't match up, meaning the redirect was from a different authorization
									// request.
									Err(InstalledAppError::MismatchedState)
								} else {
									// The code was successfully recieved at this point.
									let code = &params["code"];
									// Consume the shared code and shutdown signal futures and signal them.
									let mut code_tx_opt = code_tx.lock().unwrap();
									let mut shutdown_tx_opt = shutdown_tx.lock().unwrap();
									if let (Some(code_tx), Some(shutdown_tx)) = (code_tx_opt.take(), shutdown_tx_opt.take()) {
										// Send the code to the external future.
										code_tx.send(code.as_ref().to_owned()).unwrap();
										// Signal the shutdown of the server, from inside the server!
										shutdown_tx.send(()).unwrap();
										// Success
										Ok(())
									} else {
										// The server has already recieved another successful redirect with a valid code. This is
										// an unlikely error and can probably be safely ignored. The server will likely shutdown soon
										// after anyway if it has already recieved a shutdown signal.
										Err(InstalledAppError::AlreadyRecieved)
									}
								}
							};
							
							// Generate the approprate HTTP response to the result using the response generator and return it
							let response = (*response_gen)(&result);
							Ok::<_, Box<dyn std::error::Error + Send + Sync>>(response)
						}
					}))
				}
			}))
			.with_graceful_shutdown(async {
				// Close server if the shutdown signal sender is activated or cancelled
				let _ = shutdown_rx.await;
				()
			});
		
		match server.await {
			Ok(_) => {},
			Err(e) => {
				log::error!("Server did not shut down successfully: {}", e);
			}
		}
		
		let code_response = code_rx.await
			.map_err(|_| RedditError::Unknown)?;
		
		// Now that we have the code that signifies that the user authorized the app, we have to use it to retrieve
		// a token to authorize future requests with, as well as a refresh token needed to refresh the token every hour.
		let mut params = form_urlencoded::Serializer::new(String::new());
		params.append_pair("grant_type", "authorization_code");
		params.append_pair("code", &code_response);
		params.append_pair("redirect_uri", &redirect);
		let params = params.finish();
		
		let mut request = Request::builder()
			.method(Method::POST)
			.uri("https://ssl.reddit.com/api/v1/access_token/.json")
			.header(
				header::AUTHORIZATION,
				HeaderValue::from_str(&format!("Basic {}", base64::encode(&format!("{}:", id))))
					.map_err(|_e| {
						log::error!("Failed to create Authorization header");
						RedditError::Unknown
					})?,
			)
			.body(Body::from(params))
			.map_err(|_| RedditError::Unknown)?;
		self.add_user_agent_header(&mut request)?;
		
		let response: json::Value = self.json_raw_request(request).await?;
		
		if let (
			Some(expires_in),
			Some(token),
			Some(refresh_token),
			Some(_scope),
		) = (
			response.get("expires_in").and_then(|t| t.as_u64()),
			response.get("access_token").and_then(|t| t.as_str()),
			response.get("refresh_token").and_then(|t| t.as_str()),
			response.get("scope").and_then(|t| t.as_str()),
		) {
			*self.auth.write().unwrap() = Some(OAuth::InstalledApp(InstalledAppOAuth {
				id,
				redirect,
				token: token.to_owned(),
				refresh_token: refresh_token.to_owned(),
				expire_instant: Instant::now() + Duration::new(expires_in.to_string().parse::<u64>().unwrap(), 0),
			}));
			Ok(())
		} else {
			Err(RedditError::Unknown)
		}
	}
}

/// Enum that contains possible errors from a request for the OAuth Installed App type.
#[derive(Debug, Snafu, Clone)]
pub enum InstalledAppError {
	/// Got a generic error in the request
	#[snafu(display("Got an unknown error: {}", msg))]
	Error {
		/// The message included in the error
		msg: String,
	},
	/// The state string wasn't present or did not match
	#[snafu(display("The states did not match"))]
	MismatchedState,
	/// The code has already been recieved
	#[snafu(display("A code was already recieved"))]
	AlreadyRecieved,
	/// No message was ever recieved
	#[snafu(display("No message was ever recieved"))]
	NeverRecieved,
}
