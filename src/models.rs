use super::schema::shortlinks;

#[derive(Insertable, Queryable, Debug, PartialEq, Eq)]
pub struct Shortlink {
    pub shortlink: String,
    pub target: String 
}
