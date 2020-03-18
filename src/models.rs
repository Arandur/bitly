use std::borrow::Cow;

use super::schema::shortlinks;

#[derive(Insertable, Queryable, Debug, PartialEq, Eq)]
pub struct Shortlink<'a> {
    pub shortlink: String,
    pub target: Cow<'a, str>
}
