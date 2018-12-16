use failure::Error;
use json;
use App;

/// A trait representing a reddit Thing that can be deserialized from JSON
pub trait Thing {
	/// Parses the thing from json
	/// # Arguments
	/// * `data` - A reference to json data to be parsed
	/// * `app` - A reference to a reddit app. This is necessary in case more data is needed to be
	/// retrieved in order to completely parse the value
	fn from_value(data: &json::Value, app: &App) -> Result<Self, Error>
	where
		Self: Sized;
}
