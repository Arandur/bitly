#[cfg(test)]
extern crate actix_http;
#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
extern crate dotenv;
extern crate rand;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

pub mod schema;
pub mod models;

pub mod server;

use chrono::{NaiveDate, NaiveDateTime};

use diesel::Connection;
use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::result::Error::DatabaseError;
use diesel::result::DatabaseErrorKind;
use diesel::r2d2;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use std::borrow::Cow;
use std::collections::HashMap;
use std::net::IpAddr;

use models::*;

sql_function! {
    fn get_stat(name: Text) -> (Timestamp, Bigint);
}

#[cfg(not(test))]
type Conn = PgConnection;

#[cfg(test)]
type Conn = SqliteConnection;

type ConnectionManager = r2d2::ConnectionManager<Conn>;

pub type Pool = r2d2::Pool<ConnectionManager>;

#[cfg(not(test))]
fn database_url() -> String {
    dotenv::dotenv().ok();

    std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
}

#[cfg(test)]
fn database_url() -> String {
    ":memory:".to_string()
}

pub fn establish_connection() -> Pool {
    let database_url = database_url();

    let manager = r2d2::ConnectionManager::new(&database_url);

    Pool::builder()
        .build(manager)
        .expect("Error connecting to database")
}

pub fn create_shortlink(conn: &Conn, target: &str) -> Shortlink {
    use schema::canonical_shortlinks;

    let mut entry = CanonicalShortlink {
        name: random_name(),
        target: Cow::from(target)
    };

    loop {
        match diesel::insert_into(canonical_shortlinks::table)
            .values(&entry)
            .execute(conn) {
            Ok(_) => {
                // Insertion was successful
                return Shortlink {
                    name: entry.name,
                    target: entry.target.to_string()
                }
            },
            Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                // We don't know which constraint was violated, so
                // we need to check if a value exists with this target
                
                let shortlink: Option<Shortlink> = canonical_shortlinks::table
                    .filter(canonical_shortlinks::target.eq(&entry.target))
                    .first::<Shortlink>(conn)
                    .optional()
                    .expect("Database error");

                match shortlink {
                    // The target already exists; return the canonical
                    // shortlink for it
                    Some(shortlink) => return shortlink,
                    // The shortlink name already exists; generate another
                    None => entry.name = random_name()
                }
            },
            Err(e) => panic!("Database error: {}", e)
        }
    }
}

pub fn create_custom_shortlink(conn: &Conn, name: &str, target: &str) -> Option<Shortlink> {
    use schema::*;

    let entry = CustomShortlink {
        name: Cow::from(name),
        target: Cow::from(target)
    };

    conn.transaction::<_, diesel::result::Error, _>(move || {
        // Check canonical AND custom shortlinks to see if this name is in use

        #[derive(QueryableByName)]
        struct Count {
            #[sql_type = "BigInt"]
            count: i64
        }

        let count: i64 = diesel::sql_query(
            "SELECT COUNT(*) AS count FROM 
                (SELECT * FROM canonical_shortlinks UNION
                 SELECT * FROM custom_shortlinks) S
             WHERE S.name = $1")
            .bind::<Text, _>(name)
            .get_result::<Count>(conn)?.count;

        if count == 0 {
            diesel::insert_into(custom_shortlinks::table)
                .values(&entry)
                .execute(conn)?;
            Ok(Some(Shortlink {
                name: entry.name.to_string(),
                target: entry.target.to_string()
            }))
        } else {
            Ok(None)
        }
    }).expect("Database error")
}

pub fn find_target(conn: &Conn, name: &str, ip_addr: Option<IpAddr>) -> Option<String> {
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
        increment_visit(conn, name, ip_addr);
    }

    target
}

#[derive(Serialize)]
pub struct AggregateStat {
    name: String,
    created_on: NaiveDateTime,
    total_visits: i64,
    visits_per_day: HashMap<NaiveDate, i64>,
    unique_visitors: i64
}

pub fn get_stats(conn: &Conn, name: &str) -> Option<AggregateStat> {
    use schema::stats;

    let created_on: Option<NaiveDateTime> = stats::table.select(stats::created_on)
        .find(name)
        .get_result(conn)
        .optional()
        .expect("Database error");

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

    #[derive(QueryableByName)]
    struct Count {
        #[sql_type = "BigInt"]
        count: i64
    }

    let unique_visitors =
        diesel::sql_query(
            "SELECT COUNT(*) AS count FROM
                (SELECT DISTINCT ip_addr FROM visits
                 WHERE name = $1) AS temp")
            .bind::<Text, _>(name)
            .get_result::<Count>(conn)
            .expect("Database error").count;


    Some(AggregateStat {
        name: name.to_string(),
        created_on: created_on,
        total_visits: total_visits,
        visits_per_day: visits_per_day,
        unique_visitors: unique_visitors
    })
}

fn random_name() -> String {
    thread_rng().sample_iter(Alphanumeric).take(7).collect()
}

fn increment_visit(conn: &Conn, name: &str, ip_addr: Option<IpAddr>) {
    use schema::visits;

    let ip_addr = ip_addr.map(|addr| format!("{}", addr));

    diesel::insert_into(visits::table)
        .values((visits::name.eq(name), 
                 visits::ip_addr.eq(ip_addr)))
        .execute(conn)
        .unwrap();
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use actix_web::dev::ServiceResponse;
    use actix_http::Request;

    use super::*;
    use super::server::*;

    /* I'm mostly concerned about testing the somewhat complex logic
     * in the creation endpoint; everything else is very straightforward.
     */

    fn set_up_tables(conn: &Conn) {
        diesel::sql_query("
            CREATE TABLE canonical_shortlinks (
                name VARCHAR(10) PRIMARY KEY,
                target VARCHAR(2048) NOT NULL UNIQUE
            )").execute(conn).expect("Database error");
        diesel::sql_query("
            CREATE TABLE custom_shortlinks (
                name VARCHAR(128) PRIMARY KEY,
                target VARCHAR(2048) NOT NULL
            )").execute(conn).expect("Database error");
    }

    fn create_request(name: Option<&str>, target: &str) -> Request {
        test::TestRequest::post()
            .uri("/create")
            .set_json(&CreateRequest {
                name: name.map(|s| s.to_owned()),
                target: target.to_string()
            })
        .to_request()
    }

    fn parse_response<'a, R: serde::Deserialize<'a>>(resp: &'a ServiceResponse) -> R {
        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error")
        };

        serde_json::from_slice(&response_body)
            .expect("Response error")
    }

    #[actix_rt::test]
    async fn create_canonical() {
        let pool = super::establish_connection();

        set_up_tables(&pool.get().unwrap());

        let mut app = test::init_service(
            App::new()
                .data(pool)
                .service(create)).await;

        let req = create_request(None, "http://www.google.com");
        
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        let response_body: CreateResponse = parse_response(&resp);

        assert_eq!(response_body.target, "http://www.google.com");
    }

    #[actix_rt::test]
    async fn create_custom() {
        let pool = super::establish_connection();

        set_up_tables(&pool.get().unwrap());

        let mut app = test::init_service(
            App::new()
                .data(pool)
                .service(create)).await;

        let req = create_request(Some("foo"), "http://www.google.com");

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        let response_body: CreateResponse = parse_response(&resp);

        assert_eq!(response_body.name, "foo");
        assert_eq!(response_body.target, "http://www.google.com");
    }

    #[actix_rt::test]
    async fn create_canonical_subsequent_returns_first_result() {
        let pool = super::establish_connection();

        set_up_tables(&pool.get().unwrap());

        let mut app = test::init_service(
            App::new()
                .data(pool)
                .service(create)).await;

        let req = create_request(None, "http://www.google.com");
        let resp = test::call_service(&mut app, req).await;
        let response_body: CreateResponse = parse_response(&resp);
        let assigned_name = response_body.name;

        let req = create_request(None, "http://www.google.com");
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        let response_body: CreateResponse = parse_response(&resp);

        assert_eq!(response_body.name, assigned_name);
        assert_eq!(response_body.target, "http://www.google.com");
    }

    #[actix_rt::test]
    async fn create_custom_multiple_for_target() {
        let pool = super::establish_connection();

        set_up_tables(&pool.get().unwrap());

        let mut app = test::init_service(
            App::new()
                .data(pool)
                .service(create)).await;

        let req = create_request(Some("foo"), "http://www.google.com");
        test::call_service(&mut app, req).await;

        let req = create_request(Some("bar"), "http://www.google.com");
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn create_custom_fail_canonical_name() {
        let pool = super::establish_connection();

        set_up_tables(&pool.get().unwrap());

        let mut app = test::init_service(
            App::new()
                .data(pool)
                .service(create)).await;

        let req = create_request(None, "http://www.google.com");
        let resp = test::call_service(&mut app, req).await;
        let response_body: CreateResponse = parse_response(&resp);
        let assigned_name = response_body.name;

        let req = create_request(Some(&assigned_name), "http://www.google.com");
        let resp = test::call_service(&mut app, req).await;

        assert!(!resp.status().is_success());
    }

    #[actix_rt::test]
    async fn create_custom_fail_custom_name() {
        let pool = super::establish_connection();

        set_up_tables(&pool.get().unwrap());

        let mut app = test::init_service(
            App::new()
                .data(pool)
                .service(create)).await;

        let req = create_request(Some("foo"), "http://www.google.com");
        test::call_service(&mut app, req).await;

        let req = create_request(Some("foo"), "http://www.google.com");
        let resp = test::call_service(&mut app, req).await;

        assert!(!resp.status().is_success());
    }
}
