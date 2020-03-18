use chrono::{DateTime, Utc};

use diesel::sql_types::Text;

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
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Text"]
    pub target: String
}

#[derive(Insertable, Queryable, Debug, PartialEq, Eq)]
pub struct Stats {
    created_on: DateTime<Utc>,
    visits: i32
    // TODO: visits per day
}
