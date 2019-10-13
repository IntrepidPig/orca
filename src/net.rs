use hyper::{
	header::{self, HeaderValue},
	Request, Response, Body,
};
use futures::{
	TryStreamExt,
};

use crate::{
	net::{
		auth::OAuth,
	},
	Reddit, RedditError,
};

pub mod auth;

impl Reddit {
	/// Adds a user-agent header to `request` fitting for the current API client
	pub fn add_user_agent_header(&self, request: &mut Request<Body>) -> Result<(), RedditError> {
		request.headers_mut().insert(header::USER_AGENT, HeaderValue::from_str(&*self.user_agent.read().unwrap())
			.map_err(|_e| RedditError::BadUserAgent)?);
		
		Ok(())
	}
	
	/// Adds an authorization header to `request` fitting for the current API client
	pub fn add_auth_header(&self, request: &mut Request<Body>) -> Result<(), RedditError> {
		match &*self.auth.read().unwrap() {
			Some(OAuth::Script(script)) => {
				request.headers_mut().insert(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", script.token)).unwrap());
			},
			Some(OAuth::InstalledApp(installed)) => {
				request.headers_mut().insert(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", installed.token)).unwrap());
			},
			None => {},
		}
		
		Ok(())
	}
	
	/// Send a request with proper authorization and user-agent headers, and attempt to parse the response as JSON.
	pub async fn json_request<T: serde::de::DeserializeOwned>(&self, request: Request<Body>) -> Result<T, RedditError> {
		let mut req = request;
		self.add_user_agent_header(&mut req)?;
		self.add_auth_header(&mut req)?;
		self.json_raw_request(req).await
	}
	
	/// Send a request with no special authorization or user-agent headers, and attempt to parse the response as JSON.
	pub async fn json_raw_request<T: serde::de::DeserializeOwned>(&self, request: Request<Body>) -> Result<T, RedditError> {
		let response = self.send_raw_request(request).await?;
		let body = response.into_body();
		let chunk = body.try_concat().await
			.map_err(|e| {
				log::error!("Failed to read HTTP response: {}", e);
				RedditError::Unknown
			})?;
		let bytes = chunk.into_bytes();
		let text = std::str::from_utf8(bytes.as_ref())
			.map_err(|_e| {
				log::error!("Got a response that wasn't valid UTF-8");
				RedditError::Unknown
			})?;
		let data: T = Reddit::parse_json(text)
			.map_err(|e| {
				log::error!("Got invalid JSON response from reddit: '{}'", text);
				e
			})?;
		Ok(data)
	}
	
	/// Send a request with proper authorization and user-agent headers.
	pub async fn send_request(&self, request: Request<Body>) -> Result<Response<Body>, RedditError> {
		let mut req = request;
		self.add_user_agent_header(&mut req)?;
		self.add_auth_header(&mut req)?;
		self.send_raw_request(req).await
	}
	
	/// Send a request with no special authorization or user-agent headers.
	pub async fn send_raw_request(&self, request: Request<Body>) -> Result<Response<Body>, RedditError> {
		log::trace!("Sending request: {:?}", request);
		let response = self.client.request(request).await
			.map_err(|e| RedditError::HttpError { source: e })?;
		log::trace!("Got response: {:?}", response);
		Ok(response)
	}
}
