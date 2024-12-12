use actix_web::{web, Scope};

mod task_1 {
    use std::{collections::HashMap, str::FromStr};

    use actix_web::{http::header::{self}, post, web, HttpResponse};
    use cargo_manifest::Manifest;
    use serde::{Deserialize, Serialize};
    use toml::Value;

    enum AvailableTypes {
        Yaml,
        Json,
        Toml
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct SantaManifest {
        package: HashMap<String, Value>,
    }
    impl SantaManifest {
        fn new(str: String, content_type: AvailableTypes) -> Result<Self, HttpResponse> {
            let new = match content_type {
                AvailableTypes::Json => if let Ok(santa_manifest) = serde_json::from_str::<SantaManifest>(&str) {
                    Some(santa_manifest)
                } else {
                    None
                },
                AvailableTypes::Yaml => if let Ok(santa_manifest) = serde_yaml::from_str::<SantaManifest>(&str) {
                    Some(santa_manifest)
                } else {
                    None
                },
                AvailableTypes::Toml => {
                    if Manifest::from_str(&str).is_err() {
                        None
                    } else {
                        if let Ok(santa_manifest) = toml::from_str::<SantaManifest>(&str) {
                            Some(santa_manifest)
                        } else {
                            None
                        }
                    }
                },
            };

            if let None = new {
                return Err(HttpResponse::BadRequest().body("Invalid manifest"));
            }

            Ok(new.unwrap())
        }
        fn validate(&self) -> HttpResponse {
            if self.valid_manifest() == false {
                return HttpResponse::BadRequest().body("Invalid manifest")
            }

            if self.keyword_received() == false {
                return HttpResponse::BadRequest().body("Magic keyword not provided");
            }

            if self.orders_path_received() == false {
                return HttpResponse::NoContent().finish();
            }

            let orders = self.mount_orders();

            if orders.is_empty() {
                return HttpResponse::NoContent().finish();
            }

            HttpResponse::Ok().body(orders)
        }
        fn valid_manifest(&self) -> bool {
            if let Ok(toml) = toml::to_string::<SantaManifest>(&self) {
                Manifest::from_str(&toml).is_ok()
            } else {
                false
            }
        }
        fn keyword_received(&self) -> bool {
            if let None = self.package.get("keywords") {
                return false;
            }

            for key in self.package["keywords"].as_array().unwrap() {
                if key.as_str().unwrap() == "Christmas 2024".to_string() {
                    return true;
                }
            }
            return false;
        }
        fn orders_path_received(&self) -> bool {
            self.package.get("metadata").and_then(|meta| meta.get("orders")).is_some()
        }
        fn mount_orders(&self) -> String {
            let mut orders = String::new();
            self.package["metadata"]["orders"].as_array().unwrap().iter().for_each(|order| {
                match (&order.get("item"), &order.get("quantity")) {
                    (Some(Value::String(item)), Some(Value::Integer(quant))) => orders.push_str(&format!("{}: {}\n", item.replace("\"", ""), quant)),
                    _ => ()
                }
            });
            orders.trim().to_string()
        }
    }

    #[post("/manifest")]
    async fn manifest(header: web::Header<header::ContentType>, bytes: web::Bytes) -> HttpResponse {
        let bytes_str = String::from_utf8(bytes.to_vec()).unwrap();
        let santa_manifest = match header.as_ref() {
            "application/toml" => SantaManifest::new(bytes_str, AvailableTypes::Toml),
            "application/yaml" => SantaManifest::new(bytes_str.clone(), AvailableTypes::Yaml),
            "application/json" => SantaManifest::new(bytes_str.clone(), AvailableTypes::Json),
            _ => return HttpResponse::UnsupportedMediaType().finish()
        };

        if let Err(err) = santa_manifest {
            return err;
        }

        let santa = santa_manifest.unwrap();
        santa.validate()
    }
}

pub fn scope() -> Scope {
    web::scope("/5")
        .service(task_1::manifest)
}