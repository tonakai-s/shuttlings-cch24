use std::{sync::Mutex, time::Duration};

use leaky_bucket::RateLimiter;
use shuttlings_cch24::challenges::{self, day_9::MilkBucket};

use actix_web::web::{self, ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let milk_bucket = web::Data::new(MilkBucket {
        bucket: Mutex::new(RateLimiter::builder().max(5).initial(5).interval(Duration::from_secs(1)).build())
    });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(challenges::intro::seek);
        cfg.service(challenges::day_2::scope());
        cfg.service(challenges::day_5::scope());
        cfg.service(challenges::day_9::scope()).app_data(milk_bucket);
    };

    Ok(config.into())
}