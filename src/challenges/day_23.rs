use actix_web::{web, Scope};

mod christmas_dinner {
    use std::{collections::HashMap, error::Error, io::{self, Read}, iter};

    use actix_multipart::form::{tempfile::TempFile, MultipartForm};
    use actix_web::{get, post, web, HttpResponse};
    use serde::{Deserialize, Serialize};
    use tera::{self, Context, Tera};
    use toml::Value;

    #[get("/star")]
    async fn lit_star() -> HttpResponse {
        HttpResponse::Ok().body(include_str!("../../assets/lit-star.html"))
    }

    #[get("/present/{color}")]
    async fn present_color(tera: web::Data<Tera>, color: web::Path<String>) -> HttpResponse {
        let (curr_color, next_color) = match color.as_str() {
            "red" => ("red", "blue"),
            "blue" => ("blue", "purple"),
            "purple" => ("purple", "red"),
            _ => return HttpResponse::ImATeapot().body("I'm a teapot")
        };

        let mut context = Context::new();
        context.insert("curr_color", curr_color);
        context.insert("next_color", next_color);

        HttpResponse::Ok().body(tera.render("present.html", &context).unwrap())
    }

    #[derive(Serialize, Deserialize)]
    struct OrnamentState {
        state: String,
        n: String
    }
    #[get("/ornament/{state}/{n}")]
    async fn ornament(tera: web::Data<Tera>, ornament_state: web::Path<OrnamentState>) -> HttpResponse {
        let (state_class, state_reverse) = match ornament_state.state.as_str() {
            "on" => (" on", "off"),
            "off" => ("", "on"),
            _ => return HttpResponse::ImATeapot().body("I'm a teapot")
        };

        let mut context = Context::new();
        context.insert("state", state_class);
        context.insert("next_state", state_reverse);
        context.insert("id", &ornament_state.n);

        HttpResponse::Ok().body(
            tera.render("ornament.html", &context).unwrap()
        )
    }

    #[derive(Debug, MultipartForm)]
    struct FormUpload {
        #[multipart(limit = "100MB")]
        lockfile: TempFile
    }
    impl FormUpload {
        fn lockfile_content(&mut self) -> Result<String, io::Error> {
            let mut content = String::new();
            match self.lockfile.file.read_to_string(&mut content) {
                Err(e) => Err(e),
                Ok(_) => Ok(content)
            }
        }
        fn checksums(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
            let content  = self.lockfile_content()?;
            let parsed = toml::from_str::<HashMap<String, Value>>(&content)?;

            let package = match parsed.get("package") {
                None => return Err(From::from("Package path was not found.")),
                Some(p) => p.as_array().unwrap()
            };
            let mut checksums: Vec<String> = vec![];
            for p in package.iter() {
                let table = p.as_table().unwrap();
                let checksum = match table.get("checksum") {
                    None => continue,
                    Some(c) => c
                };
                let checksum = match checksum.as_str() {
                    None => return Err(From::from("Not a string")),
                    Some(c) => c
                };
                let checksum_contains_only_valid_hex = checksum
                    .to_lowercase()
                    .chars()
                    .all(|c| matches!(c, '0'..='9') || matches!(c, 'a'..='f'));

                if checksum.len() < 10 || checksum_contains_only_valid_hex == false {
                    return Err(From::from("Invalid checksum"));
                }
                checksums.push(checksum.to_string());
            }
            Ok(checksums)
        }
    }

    #[post("/lockfile")]
    async fn lockfile(MultipartForm(mut form): MultipartForm<FormUpload>) -> HttpResponse {
        let checksums = match form.checksums() {
            Err(e) => {
                if e.to_string() == "Invalid checksum" {
                    return HttpResponse::UnprocessableEntity().finish();
                }

                return HttpResponse::BadRequest().finish();
            },
            Ok(c) => c
        };
        let mut divs: Vec<_> = vec![];
        for checksum in checksums.iter() {
            let color = &checksum[..6];
            let top = i64::from_str_radix(&checksum[6..8], 16).unwrap();
            let left = i64::from_str_radix(&checksum[8..10], 16).unwrap();

            divs.push(format!("<div style=\"background-color:#{};top:{}px;left:{}px;\"></div>", color, top, left));
        }

        HttpResponse::Ok().body(divs.join("\n"))
    }
}

pub fn scope() -> Scope {
    web::scope("/23")
        .service(christmas_dinner::lit_star)
        .service(christmas_dinner::present_color)
        .service(christmas_dinner::ornament)
        .service(christmas_dinner::lockfile)
}
