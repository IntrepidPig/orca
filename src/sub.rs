pub enum Sort {
	Hot,
	New,
	Rising,
	Top(SortTime),
	Controversial(SortTime)
}

impl Sort {
	pub fn param<'a>(self) -> Vec<(&'a str, &'a str)> {
		use self::Sort::*;
		match self {
			Hot => {
				vec![("sort", "hot")]
			},
			New => {
				vec![("sort", "new")]
			},
			Rising => {
				vec![("sort", "rising")]
			},
			Top(sort) => {
				vec![("sort", "top"), sort.param()]
			},
			Controversial(sort) => {
				vec![("sort", "controversial"), sort.param()]
			}
		}
	}
}

pub enum SortTime {
	Hour,
	Day,
	Week,
	Month,
	Year,
	All
}

impl SortTime {
	pub fn param<'a>(self) -> (&'a str, &'a str) {
		use self::SortTime::*;
		("t", match self {
			Hour => "hour",
			Day => "day",
			Week => "week",
			Month => "month",
			Year => "year",
			All => "all"
		})
	}
}