use actix_web::{get, http::{header, StatusCode}, HttpResponse};

#[get("/-1/seek")]
pub async fn seek() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((header::LOCATION, "https://www.youtube.com/watch?v=9Gc4QTqslN4"))
        .status(StatusCode::FOUND)
        .finish()
}