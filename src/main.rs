use std::{fs::File, sync::Mutex, time::Duration};

use actix_files::Files;
use leaky_bucket::RateLimiter;
use shuttle_runtime::SecretStore;
use shuttlings_cch24::challenges::{self, day_12::Board, day_9::MilkBucket, day_19::Paginator};

use actix_web::web::{self, ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;
use tera::Tera;
use tokio::sync::watch::channel;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!("db/migrations")
        .run(&pool)
        .await
        .expect("Failed on running the migrations.");

    let jwt_secret = secrets.get("PUB_PEM").expect("Unable to read PUB_PEM secret").as_bytes().to_vec();
    let milk_bucket = web::Data::new(MilkBucket {
        bucket: Mutex::new(RateLimiter::builder().max(5).initial(5).interval(Duration::from_secs(1)).build())
    });
    let milk_cookie_board = web::Data::new(Mutex::new(Board::new()));
    let paginator = web::Data::new(Mutex::new(Paginator::new()));
    let tera = match Tera::new("./assets/*.html") {
        Err(e) => {
            println!("Parsing error: {:?}", e);
            panic!();
        },
        Ok(t) => t
    };

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(Files::new("/assets", "assets").show_files_listing());
        cfg.service(challenges::intro::seek);
        cfg.service(challenges::day_2::scope());
        cfg.service(challenges::day_5::scope());
        cfg.service(challenges::day_9::scope())
            .app_data(milk_bucket);
        cfg.service(challenges::day_12::scope())
            .app_data(milk_cookie_board);
        cfg.service(challenges::day_16::scope())
            .app_data(web::Data::new(jwt_secret));
        cfg.service(challenges::day_19::scope())
            .app_data(web::Data::new(pool))
            .app_data(paginator.clone());
        cfg.service(challenges::day_23::scope())
            .app_data(web::Data::new(tera));
    };

    Ok(config.into())
}
