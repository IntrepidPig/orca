/// An enum containing possible errors from a request to reddit
#[derive(Debug, Fail)]
pub enum RedditError {
	/// The requested resource was not found
	#[fail(display = "Requested resource {} was not found", request)]
	NotFound {
		/// The requested resource
		request: String
	},
	/// The requested resource is forbidden
	#[fail(display = "Requested resource {} is forbidden", request)]
	Forbidden {
		/// The requested resource
		request: String
	},
	/// Recieved a response that was unexpected
	#[fail(display = "\nSent request {}, got unexpected reponse {}\n", request, response)]
	BadResponse {
		/// The request that was sent
		request: String,
		/// The response that was recieved
		response: String
	},
	/// A request was sent that was incorrect
	#[fail(display = "\nAttempted incorrect request {} got response {}\n", request, response)]
	BadRequest {
		/// The request that was sent
		request: String,
		/// The response that was recieved
		response: String
	},
	/// Authorization failed
	#[fail(display = "Failed to authorize")]
	AuthError
}

/// An error representing a json value that could not be parsed as a certain struct
#[derive(Debug, Fail)]
#[fail(display = "Could not parse json {} as {}\n", json, thing_type)]
pub struct ParseError {
	/// The type the json was attempted to be parsed as
	pub thing_type: String,
	/// The json that was attempted to be parsed
	pub json: String,
}
