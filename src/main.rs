use std::{fs::{self,  File}, io::Read, sync::Mutex, time::Duration};

use leaky_bucket::RateLimiter;
use shuttle_runtime::SecretStore;
use shuttlings_cch24::challenges::{self, day_12::Board, day_9::MilkBucket};

use actix_web::web::{self, ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let jwt_secret = secrets.get("PUB_PEM").expect("Unable to read PUB_PEM secret").as_bytes().to_vec();

    let milk_bucket = web::Data::new(MilkBucket {
        bucket: Mutex::new(RateLimiter::builder().max(5).initial(5).interval(Duration::from_secs(1)).build())
    });
    let milk_cookie_board = web::Data::new(Mutex::new(Board::new()));

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(challenges::intro::seek);
        cfg.service(challenges::day_2::scope());
        cfg.service(challenges::day_5::scope());
        cfg.service(challenges::day_9::scope()).app_data(milk_bucket);
        cfg.service(challenges::day_12::scope()).app_data(milk_cookie_board);
        cfg.service(challenges::day_16::scope()).app_data(web::Data::new(jwt_secret));
    };

    Ok(config.into())
}
