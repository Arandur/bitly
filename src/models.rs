use chrono::NaiveDateTime;

use diesel::sql_types::*;

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

#[derive(Queryable, Debug, PartialEq, Eq)]
pub struct Stat {
    name: String,
    created_on: NaiveDateTime
}

#[derive(Queryable, Debug, PartialEq, Eq)]
pub struct Visit {
    id: i32,
    name: String,
    visit: NaiveDateTime
}

#[derive(QueryableByName, Debug, PartialEq, Eq)]
pub struct AggregateVisits {
    #[sql_type = "Timestamp"]
    pub visit_date: NaiveDateTime,
    #[sql_type = "Bigint"]
    pub visit_count: i64
}
