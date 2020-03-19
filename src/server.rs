use actix_web::{get, post, web, HttpRequest, HttpResponse, ResponseError, Responder};
use actix_web::http;
use actix_web::http::StatusCode;

use serde::{Serialize, Deserialize};

use url::Url;

use std::fmt::{self, Display, Formatter};

use super::*;

#[derive(Serialize, Deserialize)]
pub struct CreateRequest {
    pub name: Option<String>,
    pub target: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateResponse {
    pub name: String,
    pub target: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CreateError {
    ShortlinkAlreadyExists(String),
    InvalidTarget(String)
}

impl Display for CreateError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let err_json = serde_json::to_string(self).unwrap();
        write!(f, "{}", err_json)
    }
}

impl ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateError::ShortlinkAlreadyExists(_) => StatusCode::CONFLICT,
            CreateError::InvalidTarget(_) => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            CreateError::ShortlinkAlreadyExists(name) =>
                HttpResponse::build(StatusCode::CONFLICT).json(json!({
                    "code": 409,
                    "msg": format!("Shortlink already exists: \"{}\"", name),
                    "name": name.to_string()
                })),
            CreateError::InvalidTarget(target) => 
                HttpResponse::build(StatusCode::BAD_REQUEST).json(json!({
                    "code": 400,
                    "msg": format!("Invalid target: \"{}\"", target),
                    "target": target.to_string()
                }))
        }
    }
}

#[post("/create")]
async fn create(pool: web::Data<Pool>, request: web::Json<CreateRequest>) -> impl Responder {
    let conn = pool.get().expect("Could not connect to database");

    // Check that the target is a valid URL
    match Url::parse(&request.target) {
        Ok(url) => url,
        Err(_) => return Err(CreateError::InvalidTarget(request.target.clone()))
    };

    let result = match &request.name {
        Some(name) => create_custom_shortlink(&conn, &name, &request.target),
        None => Some(create_shortlink(&conn, &request.target))
    };

    if let Some(result) = result {
        let response = CreateResponse {
            name: result.name,
            target: request.target.clone()
        };

        Ok(web::Json(response))
    } else {
        Err(CreateError::ShortlinkAlreadyExists(request.name.clone().unwrap()))
    }
}

#[get("/{name}")]
async fn load(
    req: HttpRequest, pool: web::Data<Pool>, 
    name: web::Path<String>) -> impl Responder {
    let conn = pool.get().expect("Could not connect to database");

    let peer_addr = req.peer_addr().map(|addr| addr.ip());

    if let Some(target) = find_target(&conn, &name, peer_addr) {
        HttpResponse::SeeOther()
            .header(http::header::LOCATION, target)
            .finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/stats/{name}")]
async fn stats(pool: web::Data<Pool>, name: web::Path<String>) -> impl Responder {
    let conn = pool.get().expect("Could not connect to database");

    if let Some(stats) = get_stats(&conn, &name) {
        Ok(web::Json(stats))
    } else {
        Err(HttpResponse::NotFound().finish())
    }
}
