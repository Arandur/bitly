extern crate actix_web; 
extern crate actix_rt;
extern crate bitly;

#[macro_use]
extern crate diesel_migrations;

use actix_web::{App, HttpServer};

use bitly::server::*;

embed_migrations!("migrations/postgres");

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let pool = bitly::establish_connection();

    let connection = pool.get().expect("Could not connect to database");

    embedded_migrations::run(&connection).expect("Could not run migrations");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(create)
            .service(load)
            .service(stats)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
