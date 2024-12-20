use actix_web::{web, Scope};

mod gift {
    use actix_web::{cookie::Cookie, get, post, web, HttpRequest, HttpResponse};
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Gift {
        content: String,
        exp: u64
    }
    impl Gift {
        fn new(content: String) -> Self {
            Gift {
                content,
                exp: 10000000000
            }
        }
    }

    #[post("/wrap")]
    async fn pack(bytes: web::Bytes) -> HttpResponse {
        const SECRET: &str = "Test";
        let json = Gift::new(String::from_utf8(bytes.to_vec()).unwrap());
        let token = encode(&Header::default(), &json, &EncodingKey::from_secret(SECRET.as_ref())).unwrap();
        let decoded = decode::<Gift>(&token, &DecodingKey::from_secret(&SECRET.as_ref()), &Validation::default());
        println!("Decode on pack: {:?}", decoded);
        HttpResponse::Ok().cookie(Cookie::new("gift", token)).finish()
    }

    #[get("/unwrap")]
    async fn unpack(req: HttpRequest) -> HttpResponse {
        let gift_jwt = req.cookie("gift");
        if gift_jwt.is_none() {
            return HttpResponse::BadRequest().finish();
        }

        const SECRET: &str = "Test";
        println!("Received JWT: {:?}", gift_jwt.clone().unwrap().to_string().replace("gift=", ""));
        let token = decode::<Gift>(&gift_jwt.unwrap().to_string(), &DecodingKey::from_secret(&SECRET.as_ref()), &Validation::default());
        if token.is_err() {
            println!("Message err: {:?}", token);
            return HttpResponse::InternalServerError().finish();
        }

        HttpResponse::Ok().body(token.unwrap().claims.content)
    }
}

pub fn scope() -> Scope {
    web::scope("/16")
        .service(gift::pack)
        .service(gift::unpack)
}