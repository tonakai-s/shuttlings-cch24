use shuttlings_cch24::challenges;

use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(challenges::intro::seek);
        cfg.service(challenges::day_2::scope());
    };

    Ok(config.into())
}