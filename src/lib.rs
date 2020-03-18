#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate rand;
#[macro_use]
extern crate serde;

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

use models::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_shortlink(conn: &PgConnection, target: &str) -> Shortlink {
    use schema::canonical_shortlinks;

    let mut entry = CanonicalShortlink {
        name: random_name(),
        target: Cow::from(target)
    };

    loop {
        match diesel::insert_into(canonical_shortlinks::table)
            .values(&entry)
            .on_conflict(canonical_shortlinks::target)
            .do_nothing()
            .get_result(conn)
            .optional() {
            Ok(Some(shortlink)) => return shortlink,
            Ok(None) => {
                return canonical_shortlinks::table.filter(canonical_shortlinks::target.eq(entry.target))
                    .first(conn)
                    .expect("Database error");
            },
            Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => entry.name = random_name(),
            Err(e) => panic!("Database error: {}", e)
        }
    }
}

pub fn create_custom_shortlink(conn: &PgConnection, name: &str, target: &str) -> Option<Shortlink> {
    use schema::*;

    let entry = CustomShortlink {
        name: Cow::from(name),
        target: Cow::from(target)
    };

    conn.transaction(move || {
        // Check canonical AND custom shortlinks to see if this name is in use

        // NOTE: This would be more convenient as a UNION query, but I couldn't
        // fiure out how to get Diesel to play nicely with that. Further
        // research is required.

        match canonical_shortlinks::table
            .find(name)
            .count()
            .get_result(conn) {
            Ok(0) => (),
            Ok(_) => return Ok(None),
            Err(e) => return Err(e)
        }

        match custom_shortlinks::table
            .find(name)
            .count()
            .get_result(conn) {
            Ok(0) => (),
            Ok(_) => return Ok(None),
            Err(e) => return Err(e)
        }

        diesel::insert_into(custom_shortlinks::table)
            .values(&entry)
            .get_result(conn)
            .optional()
    }).expect("Database error")
}

pub fn find_target(conn: &PgConnection, name: &str) -> Option<String> {
    use schema::*;

    let target = canonical_shortlinks::table
        .find(name)
        .get_result(conn)
        .optional()
        .expect("Database error")
        .map(|entry: Shortlink| entry.target)
        .or_else(|| {
            custom_shortlinks::table
                .find(name)
                .get_result(conn)
                .optional()
                .expect("Database error")
                .map(|entry: Shortlink| entry.target)
        });

    if target.is_some() {
        increment_visit(conn, name);
    }

    target
}

pub fn get_stats(conn: &PgConnection, name: &str) -> Option<Stats> {
    use schema::stats;

    stats::table
        .find(name)
        .get_result(conn)
        .optional()
        .expect("Database error")
}

fn random_name() -> String {
    thread_rng().sample_iter(Alphanumeric).take(7).collect()
}

fn increment_visit(conn: &PgConnection, name: &str) {
    use schema::stats;

    diesel::update(stats::table)
        .filter(stats::name.eq(name))
        .set(stats::visits.eq(stats::visits + 1))
        .get_results::<Stats>(conn)
        .unwrap();
}
