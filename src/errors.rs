use failure::{Fail, Error, err_msg};
use hyper::{Request, Response};

use json::{to_string_pretty, Value, Error as JsonError};

#[derive(Debug, Fail)]
#[fail(display = "The requested resource could not be found")]
pub struct NotFound {}

#[derive(Debug, Fail)]
#[fail(display = "Got an unexpected reponse:\n{}", response)]
pub struct BadResponse {
	pub response: String,
}

#[derive(Debug, Fail)]
#[fail(display = "Failed to execute the request")]
pub struct BadRequest {}

#[derive(Debug, Fail)]
pub enum RedditError {
	#[fail(display = "Requested resource {} was not found", request)]
	NotFound { request: String },
	#[fail(display = "Requested resource {} is forbidden", request)]
	Forbidden { request: String },
	#[fail(display = "\nSent request {}, got unexpected reponse {}\n", request, response)]
	BadResponse { request: String, response: String },
	#[fail(display = "\nAttempted incorrect request {} got response {}\n", request, response)]
	BadRequest { request: String, response: String },
}

#[derive(Debug, Fail)]
#[fail(display = "Could not parse json {} as {}", json, thing_type)]
pub struct ParseError {
	pub thing_type: String,
	pub json: Value,
}
