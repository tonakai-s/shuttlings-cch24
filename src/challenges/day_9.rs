use std::sync::Mutex;

use actix_web::{web, Scope};
use leaky_bucket::RateLimiter;

pub struct MilkBucket {
    pub bucket: Mutex<RateLimiter>
}

mod milk {
    use std::{mem, time::Duration};

    use actix_web::{post, web, HttpRequest, HttpResponse};
    use leaky_bucket::RateLimiter;
    use serde::{Deserialize, Serialize};
    use crate::challenges::day_9::MilkBucket;

    const US_LITERS_PER_GALLON: f32 = 3.785411784;
    const UK_LITRES_PER_PINT: f32 = 0.56826125;
    #[derive(Debug, Deserialize, Serialize)]
    enum Units {
        #[serde(rename = "liters")]
        Liters(f32),
        #[serde(rename = "gallons")]
        Gallons(f32),
        #[serde(rename = "litres")]
        Litres(f32),
        #[serde(rename = "pints")]
        Pints(f32)
    }

    #[post("/milk")]
    async fn withdraw_milk(milk_bucket_state: web::Data<MilkBucket>, payload: web::Payload, req: HttpRequest) -> HttpResponse {
        if milk_bucket_state.bucket.lock().unwrap().try_acquire(1) == false {
            return HttpResponse::TooManyRequests().body("No milk available\n");
        }

        let content_type = req.headers().get("content-type");
        if content_type.is_none() || content_type.unwrap() != "application/json" {
            return HttpResponse::Ok().body("Milk withdrawn\n");
        }

        let body = payload.to_bytes().await.unwrap().to_vec();
        if let Ok(json) = serde_json::from_slice::<Units>(&body) {
            match json {
                Units::Gallons(g) => return HttpResponse::Ok().json(Units::Liters(g * US_LITERS_PER_GALLON)),
                Units::Liters(l) => return HttpResponse::Ok().json(Units::Gallons(l / US_LITERS_PER_GALLON)),
                Units::Pints(p) => return HttpResponse::Ok().json(Units::Litres(p * UK_LITRES_PER_PINT)),
                Units::Litres(l) => return HttpResponse::Ok().json(Units::Pints(l / UK_LITRES_PER_PINT))
            }
        }

        HttpResponse::BadRequest().finish()
    }

    #[post("/refill")]
    async fn refill(milk_bucket_state: web::Data<MilkBucket>) -> HttpResponse {
        let mut state = milk_bucket_state.bucket.lock().unwrap();
        let _ = mem::replace(&mut *state, RateLimiter::builder().max(5).initial(5).interval(Duration::from_secs(1)).build());
        HttpResponse::Ok().finish()
    }
}

pub fn scope() -> Scope {
    web::scope("/9")
        .service(milk::withdraw_milk)
        .service(milk::refill)
}