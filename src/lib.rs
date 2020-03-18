#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate rand;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::Error::DatabaseError;
use diesel::result::DatabaseErrorKind;

use dotenv::dotenv;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use std::borrow::Cow;
use std::env;

use models::Shortlink;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_shortlink<'a>(conn: &PgConnection, target: &'a str) -> Shortlink<'a> {
    use schema::shortlinks;

    let mut entry = Shortlink {
        shortlink: random_shortlink(),
        target: Cow::from(target)
    };

    loop {
        match diesel::insert_into(shortlinks::table)
            .values(&entry)
            .get_result(conn) {
            Ok(shortlink) => return shortlink,
            Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => entry.shortlink = random_shortlink(),
            Err(e) => panic!("Database error: {}", e)
        }
    }
}

fn random_shortlink() -> String {
    thread_rng().sample_iter(Alphanumeric).take(7).collect()
}
