use actix_web::{web, Scope};

mod task_1 {
    use actix_web::{post, web, HttpResponse, Responder};
    use toml::{Table, Value};

    #[post("/manifest")]
    async fn manifest(bytes: web::Bytes) -> impl Responder {
        let toml = String::from_utf8(bytes.to_vec()).unwrap().parse::<Table>().unwrap();
        println!("Toml: {:#?}", toml);

        let mut valid_orders = String::new();
        if let None = &toml.get("package")
                .and_then(|pkg| pkg.get("metadata"))
                .and_then(|meta| meta.get("orders"))
        {
            return HttpResponse::NoContent().finish();
        }

        match &toml["package"]["metadata"]["orders"] {
            Value::Array(orders) => {
                orders.iter().for_each(|order| {
                    match (&order.get("item"), &order.get("quantity")) {
                        (Some(Value::String(item)), Some(Value::Integer(quant))) => valid_orders.push_str(&format!("{}: {}\n", item.replace("\"", ""), quant)),
                        _ => ()
                    }
                })
            },
            _ => ()
        };

        if valid_orders.is_empty() {
            return HttpResponse::NoContent().finish();
        }

        valid_orders = valid_orders.trim().to_string();
        HttpResponse::Ok().body(valid_orders)
    }
}

pub fn scope() -> Scope {
    web::scope("/5")
        .service(task_1::manifest)
}