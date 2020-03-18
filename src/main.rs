extern crate actix_web; 
extern crate actix_rt;
extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_json;

use actix_web::{get, post, web, App, HttpServer, HttpResponse, ResponseError, Responder};
use actix_web::http::{self, StatusCode};
use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

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
    CustomShortlinkAlreadyExists(String)
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
            CreateError::CustomShortlinkAlreadyExists(shortlink) =>
                HttpResponse::build(StatusCode::CONFLICT).json(json!({
                    "code": 309,
                    "msg": format!("Custom shortlink already exists: \"{}\"", shortlink),
                    "name": shortlink.to_string()
                }))
        }
    }
}

#[post("/create")]
async fn create(request: web::Json<CreateRequest>) -> Result<web::Json<CreateResponse>, CreateError> {
    let conn = bitly::establish_connection();

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
        Err(CreateError::CustomShortlinkAlreadyExists(request.name.clone().unwrap()))
    }
}

#[get("/{name}")]
async fn load(name: web::Path<String>) -> HttpResponse {
    let conn = bitly::establish_connection();

    if let Some(target) = bitly::find_target(&conn, &name) {
        HttpResponse::SeeOther()
            .header(http::header::LOCATION, target)
            .finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/stats/{name}")]
async fn stats(name: web::Path<String>) -> impl Responder {
    let conn = bitly::establish_connection();

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
            .service(create)
            .service(load)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
