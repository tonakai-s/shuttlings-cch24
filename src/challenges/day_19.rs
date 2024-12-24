use actix_web::{web, Scope};
use rand::{self, distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::{chrono::{DateTime, Utc}, Uuid}};

#[derive(Deserialize)]
pub struct NewQuote {
    author: String,
    quote: String
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32
}

type PaginatorPages = Vec<(String, Vec<Quote>)>;
#[derive(Debug, Clone)]
pub struct Paginator {
    pages: PaginatorPages,
    traversed: bool
}
impl Paginator {
    pub fn new() -> Self {
        Paginator {
            pages: vec![],
            traversed: false
        }
    }
    pub fn set_pages(&mut self, quotes: Vec<Quote>) {
        quotes.chunks(3).for_each(|chunk| {
            let token: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            self.pages.push((token, chunk.to_vec()));
        });
    }
    pub fn next_page(&mut self, token: Option<String>) -> Option<PageList> {
        let quotes_pos: Option<usize> = match &token {
            None => Some(0),
            Some(token) => self.pages.iter().position(|(t, _)| { *t == *token })
        };
        if quotes_pos.is_none() {
            return None;
        }

        let next_pos = quotes_pos.unwrap().wrapping_add(1);
        Some(PageList {
            quotes: self.pages[quotes_pos.unwrap()].1.clone(),
            page: next_pos,
            next_token: match self.pages.get(next_pos) {
                None => {
                    self.traversed = true;
                    None
                },
                Some((token, _)) => Some(token.clone())
            }
        })
    }
}
#[derive(Serialize, Debug)]
pub struct PageList {
    quotes: Vec<Quote>,
    page: usize,
    next_token: Option<String>,
}

mod crud {
    use std::{mem, str::FromStr, sync::Mutex};
    use crate::challenges::day_19::{NewQuote, Quote, Paginator};

    use actix_web::{delete, get, post, put, web, HttpResponse};
    use serde::Deserialize;
    use sqlx::{postgres::{PgPool, PgQueryResult}, types::Uuid};

    #[post("/reset")]
    async fn reset(pgpool: web::Data<PgPool>) -> HttpResponse {
        let _: PgQueryResult = sqlx::query("DELETE FROM quotes").execute(pgpool.get_ref()).await.unwrap();
        HttpResponse::Ok().finish()
    }

    #[post("/draft")]
    async fn draft(pgpool: web::Data<PgPool>, json: web::Json<NewQuote>) -> HttpResponse {
        match sqlx::query_as::<_, Quote>("INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3)
            RETURNING id, author, quote, created_at, version")
            .bind(Uuid::new_v4())
            .bind(&json.author)
            .bind(&json.quote)
            .fetch_one(pgpool.get_ref()).await {
            Err(_) => HttpResponse::Ok().finish(),
            Ok(quote) => HttpResponse::Created().json(quote)
        }
    }

    #[delete("/remove/{id}")]
    async fn remove(pgpool: web::Data<PgPool>, id: web::Path<String>) -> HttpResponse {
        let uuid = Uuid::from_str(&id);
        if uuid.is_err() {
            return HttpResponse::BadRequest().finish();
        }
        match sqlx::query_as::<_, Quote>(
            "DELETE FROM quotes WHERE id = $1 RETURNING id, author, quote, created_at, version"
            ).bind(uuid.unwrap())
            .fetch_one(pgpool.get_ref()).await {
            Err(_) => HttpResponse::NotFound().finish(),
            Ok(quote) => HttpResponse::Ok().json(quote)
        }
    }

    #[get("/cite/{id}")]
    async fn cite(pgpool: web::Data<PgPool>, id: web::Path<String>) -> HttpResponse {
        let uuid = Uuid::from_str(&id);
        if uuid.is_err() {
            return HttpResponse::BadRequest().finish();
        }
        match sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
            .bind(uuid.unwrap())
            .fetch_one(pgpool.get_ref()).await {
            Err(_) => HttpResponse::NotFound().finish(),
            Ok(quote) => HttpResponse::Ok().json(quote)
        }
    }

    #[put("/undo/{id}")]
    async fn undo(pgpool: web::Data<PgPool>, id: web::Path<String>, json: web::Json<NewQuote>) -> HttpResponse {
        let uuid = Uuid::from_str(&id);
        if uuid.is_err() {
            return HttpResponse::BadRequest().finish();
        }
        let quote_id = uuid.unwrap();
        let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
            .bind(&quote_id)
            .fetch_one(pgpool.get_ref()).await;
        if quote.is_err() {
            return HttpResponse::NotFound().finish();
        }
        let version = quote.unwrap().version.wrapping_add(1);

        match sqlx::query_as::<_, Quote>("UPDATE quotes SET author = $1, quote = $2, version = $3 WHERE id = $4
            RETURNING id, author, quote, created_at, version")
            .bind(&json.author)
            .bind(&json.quote)
            .bind(version)
            .bind(&quote_id)
            .fetch_one(pgpool.get_ref()).await {
            Err(_) => HttpResponse::NotFound().finish(),
            Ok(quote) => HttpResponse::Ok().json(quote)
        }
    }

    #[derive(Debug, Deserialize)]
    struct ListParams {
        token: Option<String>
    }
    #[get("/list")]
    async fn list(
        pgpool: web::Data<PgPool>,
        paginator_state: web::Data<Mutex<Paginator>>,
        token: web::Query<ListParams>
    ) -> HttpResponse {
        let mut paginator_guard = paginator_state.lock().unwrap();

        match token.into_inner().token.clone() {
            None => {
                match sqlx::query_as::<_, Quote>("SELECT * FROM quotes ORDER BY created_at ASC")
                    .fetch_all(pgpool.get_ref()).await {
                    Err(_) => HttpResponse::InternalServerError().finish(),
                    Ok(quotes) => {
                        if paginator_guard.traversed == true || paginator_guard.pages.len() == 0 {
                            let mut paginator = Paginator::new();
                            paginator.set_pages(quotes);

                            let _ = mem::replace(&mut *paginator_guard, paginator);
                        }
                        match paginator_guard.next_page(None) {
                            None => HttpResponse::BadRequest().finish(),
                            Some(p) => HttpResponse::Ok().json(p)
                        }
                    }
                }
            },
            Some(token) => {
                if token.len() != 16 {
                    return HttpResponse::BadRequest().finish();
                }
                match paginator_guard.next_page(Some(token)) {
                    None => HttpResponse::BadRequest().finish(),
                    Some(p) => HttpResponse::Ok().json(p)
                }
            }
        }
    }
}

pub fn scope() -> Scope {
    web::scope("/19")
        .service(crud::reset)
        .service(crud::draft)
        .service(crud::remove)
        .service(crud::cite)
        .service(crud::undo)
        .service(crud::list)
}
