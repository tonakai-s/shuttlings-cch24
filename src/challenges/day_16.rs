use actix_web::{web, Scope};

mod gift {
    use std::collections::{HashMap, HashSet};

    use actix_web::{cookie::Cookie, get, post, web, HttpRequest, HttpResponse};
    use jsonwebtoken::{decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde_json::Value;

    type Gift = HashMap<String, Value>;

    #[post("/wrap")]
    async fn pack(jwt_secret: web::Data<Vec<u8>>, bytes: web::Bytes) -> HttpResponse {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_slice());
        let json_gift: Gift = serde_json::from_slice::<Gift>(&bytes)
            .expect("Unable to parse the gift on pack.");

        match encode(&Header::default(), &json_gift, &encoding_key) {
            Err(e) => {
                println!("Error on encoding the gift: {}", e);
                HttpResponse::InternalServerError().finish()
            }
            Ok(gift) => HttpResponse::Ok().cookie(Cookie::new("gift", gift)).finish()
        }
    }

    #[get("/unwrap")]
    async fn unpack(jwt_secret: web::Data<Vec<u8>>, req: HttpRequest) -> HttpResponse {
        let gift_jwt = req.cookie("gift");
        if gift_jwt.is_none() {
            return HttpResponse::BadRequest().finish();
        }

        let jwt_token = gift_jwt.unwrap().to_string().split("=").skip(1).collect::<String>();
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_slice());

        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::from(["".to_string()]);

        match decode::<Gift>(&jwt_token, &decoding_key, &validation) {
            Err(e) => {
                println!("Message err: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
            Ok(gift) => HttpResponse::Ok().json(gift.claims)
        }
    }

    #[post("/decode")]
    async fn unpack_olders(jwt_secret: web::Data<Vec<u8>>, bytes: web::Bytes) -> HttpResponse {
        let token = String::from_utf8(bytes.to_vec()).unwrap();
        let decoding_key = DecodingKey::from_rsa_pem(jwt_secret.as_slice())
            .expect("Unable to generate decode key from PEM.");

        let mut validation = Validation::default();
        validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512];
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::from(["".to_string()]);

        match decode::<Gift>(&token, &decoding_key, &validation) {
            Ok(gift) => HttpResponse::Ok().json(gift.claims),
            Err(e) => match e.kind() {
                ErrorKind::InvalidSignature => HttpResponse::Unauthorized().finish(),
                _ => HttpResponse::BadRequest().finish()
            }
        }
    }
}

pub fn scope() -> Scope {
    web::scope("/16")
        .service(gift::pack)
        .service(gift::unpack)
        .service(gift::unpack_olders)
}
