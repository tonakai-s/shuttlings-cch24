use actix_web::{get, http::{header, StatusCode}, web::ServiceConfig, HttpResponse};
use shuttle_actix_web::ShuttleActixWeb;

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello, bird!"
}

#[get("/-1/seek")]
async fn seek() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((header::LOCATION, "https://www.youtube.com/watch?v=9Gc4QTqslN4"))
        .status(StatusCode::FOUND)
        .finish()
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world);
        cfg.service(seek);
    };

    Ok(config.into())
}