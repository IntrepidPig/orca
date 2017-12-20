use failure::{Fail, Error, err_msg};

use json::{Value, Error as JsonError};

#[derive(Debug, Fail)]
#[fail(display = "The requested resource could not be found")]
pub struct NotFound {}

#[derive(Debug, Fail)]
#[fail(display = "The requested resource is forbidden")]
pub struct Forbidden {}

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
	#[fail(display = "Requested resource was not found")]
	NotFound,
	#[fail(display = "Requested resource is forbidden")]
	Forbidden,
	#[fail(display = "Got an unexpected response:\n{}", response)]
	BadResponse { response: String },
	#[fail(display = "Failed to execute the request")]
	BadRequest,
}

#[derive(Debug, Fail)]
#[fail(display = "Could not parse thing")]
pub struct ParseError {
	pub raw_json: String,
}
