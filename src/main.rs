use std::{net::Ipv4Addr, str::FromStr};

use actix_web::{get, http::{header, StatusCode}, web::{self, ServiceConfig}, HttpResponse};
use serde::Deserialize;
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

#[derive(Deserialize)]
struct SantaFromKey {
    from: String,
    key: String
}
impl SantaFromKey {
    fn dest(&self) -> String {
        let vec_key = Ipv4Addr::from_str(&self.key).unwrap().octets();
        Ipv4Addr::from_str(&self.from).unwrap().octets().iter().enumerate().map(|(i, n)| {
            n.wrapping_add(vec_key[i]).to_string()
        }).collect::<Vec<String>>().join(".")
    }
}
#[get("/2/dest")]
async fn dest(route: web::Query<SantaFromKey>) -> String {
    route.dest()
}

#[derive(Deserialize)]
struct SantaFromTo {
    from: String,
    to: String
}
impl SantaFromTo {
    fn key(&self) -> String {
        let vec_from = Ipv4Addr::from_str(&self.from).unwrap().octets();
        Ipv4Addr::from_str(&self.to).unwrap().octets().iter().enumerate().map(|(i, n)| {
            n.wrapping_sub(vec_from[i]).to_string()
        }).collect::<Vec<String>>().join(".")
    }
}
#[get("/2/key")]
async fn key(route: web::Query<SantaFromTo>) -> String {
    route.key()
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world);
        cfg.service(seek);
        cfg.service(dest);
        cfg.service(key);
    };

    Ok(config.into())
}