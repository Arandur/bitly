extern crate actix_web; 
extern crate actix_rt;
extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_json;

use actix_web::{get, post, web, App, HttpServer, HttpResponse, ResponseError, Responder};
use actix_web::http::{self, StatusCode};

use diesel::pg::PgConnection;
use diesel::r2d2::{Pool, ConnectionManager};

use serde::{Serialize, Deserialize};

use std::fmt::{Display, Formatter, Result as FmtResult};

type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Deserialize)]
struct CreateRequest {
    name: Option<String>,
    target: String
}

#[derive(Serialize)]
struct CreateResponse {
    name: String,
    target: String
}

#[derive(Debug, Serialize)]
enum CreateError {
    ShortlinkAlreadyExists(String)
}

impl Display for CreateError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let err_json = serde_json::to_string(self).unwrap();
        write!(f, "{}", err_json)
    }
}

impl ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        StatusCode::CONFLICT
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            CreateError::ShortlinkAlreadyExists(name) =>
                HttpResponse::build(StatusCode::CONFLICT).json(json!({
                    "code": 409,
                    "msg": format!("Shortlink already exists: \"{}\"", name),
                    "name": name.to_string()
                }))
        }
    }
}

#[post("/create")]
async fn create(pool: web::Data<PgPool>, request: web::Json<CreateRequest>) -> impl Responder {
    let conn = pool.get().expect("Could not connect to database");

    let result = match &request.name {
        Some(name) => bitly::create_custom_shortlink(&conn, &name, &request.target),
        None => Some(bitly::create_shortlink(&conn, &request.target))
    };

    if let Some(result) = result {
        let response = CreateResponse {
            name: result.name,
            target: result.target
        };

        Ok(web::Json(response))
    } else {
        Err(CreateError::ShortlinkAlreadyExists(request.name.clone().unwrap()))
    }
}

#[get("/{name}")]
async fn load(pool: web::Data<PgPool>, name: web::Path<String>) -> HttpResponse {
    let conn = pool.get().expect("Could not connect to database");

    if let Some(target) = bitly::find_target(&conn, &name) {
        HttpResponse::SeeOther()
            .header(http::header::LOCATION, target)
            .finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/stats/{name}")]
async fn stats(pool: web::Data<PgPool>, name: web::Path<String>) -> impl Responder {
    let conn = pool.get().expect("Could not connect to database");

    if let Some(stats) = bitly::get_stats(&conn, &name) {
        Ok(web::Json(stats))
    } else {
        Err(HttpResponse::NotFound().finish())
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .data(bitly::establish_connection())
            .service(create)
            .service(load)
            .service(stats)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
