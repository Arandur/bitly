use chrono::NaiveDateTime;

use std::borrow::Cow;

use super::schema::*;

#[derive(Insertable, Queryable, Debug, PartialEq, Eq)]
#[table_name = "canonical_shortlinks"]
pub struct CanonicalShortlink<'a> {
    pub name: String,
    pub target: Cow<'a, str>
}

#[derive(Insertable, Queryable, Debug, PartialEq, Eq)]
#[table_name = "custom_shortlinks"]
pub struct CustomShortlink<'a> {
    pub name: Cow<'a, str>,
    pub target: Cow<'a, str>
}

#[derive(Queryable, Debug, PartialEq, Eq)]
pub struct Shortlink {
    pub name: String,
    pub target: String
}

// TODO: Times should be in UTC
#[derive(Queryable, Debug, PartialEq, Eq, Serialize)]
pub struct Stats {
    name: String,
    created_on: NaiveDateTime,
    visits: i32
    // TODO: visits per day
}
