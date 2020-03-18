#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate rand;
#[macro_use]
extern crate serde;

pub mod schema;
pub mod models;

use chrono::{DateTime, NaiveDate, Utc};

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::sql_types::*;
use diesel::result::Error::DatabaseError;
use diesel::result::DatabaseErrorKind;

use dotenv::dotenv;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use std::borrow::Cow;
use std::collections::HashMap;
use std::env;

use models::*;

sql_function! {
    fn get_stat(name: Text) -> (Timestamp, Bigint);
}

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

        #[derive(QueryableByName)]
        struct Count {
            #[sql_type = "BigInt"]
            count: i64
        }

        let count: i64 = diesel::sql_query(
            "SELECT COUNT(*) FROM 
                (SELECT * FROM canonical_shortlinks UNION
                 SELECT * FROM custom_shortlinks) S
             WHERE S.name = $1")
            .bind::<Text, _>(name)
            .get_result::<Count>(conn)
            .expect("Database error").count;

        if count == 0 {
            diesel::insert_into(custom_shortlinks::table)
                .values(&entry)
                .get_result(conn)
                .optional()
        } else {
            Ok(None)
        }
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

#[derive(Serialize)]
pub struct AggregateStat {
    name: String,
    created_on: DateTime<Utc>,
    total_visits: i64,
    visits_per_day: HashMap<NaiveDate, i64>
}

pub fn get_stats(conn: &PgConnection, name: &str) -> Option<AggregateStat> {
    use schema::stats;

    let created_on = stats::table.select(stats::created_on)
        .find(name)
        .get_result(conn)
        .optional()
        .unwrap();

    if created_on.is_none() {
        return None;
    }

    let created_on = created_on.unwrap();

    // Wish I could iterate over this instead of wastefully pulling it into 
    // a vec, but that's the API I'm given...
    let visits: Vec<AggregateVisits> = 
        diesel::sql_query("SELECT * FROM get_stat($1);")
            .bind::<Text, _>(name)
            .get_results(conn)
            .unwrap();

    let total_visits = visits.iter().map(|ag| ag.visit_count).sum();

    let visits_per_day: HashMap<NaiveDate, i64> = 
        visits.into_iter()
            .map(|ag| (ag.visit_date.date(), ag.visit_count))
            .collect();

    Some(AggregateStat {
        name: name.to_string(),
        created_on: created_on,
        total_visits: total_visits,
        visits_per_day: visits_per_day
    })
}

fn random_name() -> String {
    thread_rng().sample_iter(Alphanumeric).take(7).collect()
}

fn increment_visit(conn: &PgConnection, name: &str) {
    use schema::visits;

    diesel::insert_into(visits::table)
        .values(visits::name.eq(name))
        .execute(conn)
        .unwrap();
}
