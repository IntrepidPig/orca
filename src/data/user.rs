use json::Value;

pub enum User {
	Authed(AuthUserData),
	Other(UserData),
}

pub struct AuthUserData {
	pub userdata: UserData,
	pub raw: Value,
}

pub struct UserData {
	pub comment_karma: i64,
	pub created: f64,
	pub created_utc: f64,
	pub has_subscribed: bool,
	pub has_verified_email: bool,
	pub hide_from_robots: bool,
	pub id: String,
	pub is_employee: bool,
	pub is_friend: bool,
	pub is_gold: bool,
	pub is_mod: bool,
	pub link_karma: i64,
	pub name: String,
	pub raw: Value,
}
