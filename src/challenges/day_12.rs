use actix_web::{web, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
pub enum Team {
    #[serde(rename = "milk")]
    Milk,
    #[serde(rename = "cookie")]
    Cookie
}

#[derive(Copy, Clone, Debug)]
pub enum TileState {
    Empty,
    Cookie,
    Milk
}
impl TileState {
    pub fn char(&self) -> char {
        match &self {
            TileState::Empty => '⬛',
            TileState::Cookie => '🍪',
            TileState::Milk => '🥛',

        }
    }
}
pub struct Board {
    grid: [[TileState;4];4],
    raw_representation: String
}
impl Board {
    pub fn new() -> Self {
        Board {
            grid: [[TileState::Empty;4];4],
            raw_representation: r#"⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬜⬜⬜⬜⬜
"#.to_string()
        }
    }

    pub fn grid_update(&mut self) {
        println!("On grid update: {:?}", self.grid);
        self.raw_representation = format!(
            "⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜⬜⬜⬜⬜⬜",
            self.grid[3][0].char(),self.grid[3][1].char(),self.grid[3][2].char(),self.grid[3][3].char(),
            self.grid[2][0].char(),self.grid[2][1].char(),self.grid[2][2].char(),self.grid[2][3].char(),
            self.grid[1][0].char(),self.grid[1][1].char(),self.grid[1][2].char(),self.grid[1][3].char(),
            self.grid[0][0].char(),self.grid[0][1].char(),self.grid[0][2].char(),self.grid[0][3].char(),
        );
    }

    pub fn column_winner(&self) -> Option<Team> {
        if self.grid.iter().any(|column| column.iter().all(|t| matches!(t, TileState::Cookie))) {
            return Some(Team::Cookie);
        }
        if self.grid.iter().any(|column| column.iter().all(|t| matches!(t, TileState::Milk))) {
            return Some(Team::Milk);
        }
        None
    }
}

mod day_12 {
    use std::{mem, ops::Sub, sync::Mutex};

    use actix_web::{ get, post, web, HttpResponse};
    use crate::challenges::day_12::{Board, TileState, Team};

    #[get("/board")]
    async fn board() -> HttpResponse {
        let board = Board::new();
        HttpResponse::Ok().body(board.raw_representation)
    }

    #[post("/reset")]
    async fn reset(board_state: web::Data<Mutex<Board>>) -> HttpResponse {
        let mut state = board_state.lock().unwrap();
        let _ = mem::replace(&mut *state, Board::new());
        HttpResponse::Ok().body(state.raw_representation.clone())
    }

    #[post("/place/{team}/{column}")]
    async fn place(board_state: web::Data<Mutex<Board>>, path: web::Path<(Team, u8)>) -> HttpResponse {
        let mut state = board_state.lock().unwrap();
        let (team, column) = path.into_inner();
        if column < 1 || column > 4 {
            return HttpResponse::BadRequest().finish();
        }

        if let Some(r) = state.grid[column as usize].iter().position(|t| matches!(t, TileState::Empty)) {
            println!("Row: {r}");
            println!("Grid: {:?}", state.grid[column as usize].clone());
            match team {
                Team::Cookie => state.grid[column.sub(1) as usize][r] = TileState::Cookie,
                Team::Milk => state.grid[column.sub(1) as usize][r] = TileState::Milk,
            }
        } else {
            return HttpResponse::ServiceUnavailable().finish();
        }

        state.grid_update();
        HttpResponse::Ok().body(state.raw_representation.clone())

        // if let Some(winner) = state.column_winner() {
        //     return HttpResponse::Ok().finish();
        // }

        // HttpResponse::Ok().finish()
    }
}

pub fn scope() -> Scope {
    web::scope("/12")
        .service(day_12::board)
        .service(day_12::reset)
        .service(day_12::place)
}