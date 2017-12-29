use json;
use failure::Error;
use App;

pub trait Thing {
	fn from_value(data: &json::Value, app: &App) -> Result<Self, Error>
	where
		Self: Sized;
}
