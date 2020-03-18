extern crate actix_web; extern crate actix_rt;
extern crate serde;

use actix_web::{post, web, App, HttpServer};
use serde::{Serialize, Deserialize};


#[derive(Deserialize)]
struct CreateRequest {
    shortlink: Option<String>,
    target: String
}

#[derive(Serialize)]
struct CreateResponse {
    shortlink: String,
    target: String
}

#[post("/create")]
async fn create(request: web::Json<CreateRequest>) -> actix_web::Result<web::Json<CreateResponse>> {
    let conn = bitly::establish_connection();

    let result = match &request.shortlink {
        Some(shortlink) => unimplemented!(),
        None => bitly::create_shortlink(&conn, &request.target)
    };

    let response = CreateResponse {
        shortlink: result.shortlink,
        target: result.target.into_owned()
    };

    Ok(web::Json(response))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(create)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
