use std::{error::Error, fmt::Display, ops::Sub};

use actix_web::{web, Scope};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum Team {
    #[serde(rename = "milk")]
    Milk,
    #[serde(rename = "cookie")]
    Cookie
}
impl TryFrom<String> for Team {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "cookie" => Ok(Self::Cookie),
            "milk" => Ok(Self::Milk),
            _ => Err("Invalid value")
        }
    }
}
impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Team::Cookie => write!(f, "🍪"),
            Team::Milk => write!(f, "🥛"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TileState {
    Empty,
    Cookie,
    Milk
}
impl Display for TileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TileState::Empty => write!(f, "⬛"),
            TileState::Cookie => write!(f, "🍪"),
            TileState::Milk => write!(f, "🥛"),
        }
    }
}
#[derive(Debug)]
pub struct Board {
    grid: [[TileState;4];4],
    raw_representation: String,
    winner: Option<Team>
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
"#.to_string(),
            winner: None
        }
    }

    pub fn set_position(&mut self, team: &Team, mut column: usize) -> Result<(), Box<dyn Error>> {
        column = column.sub(1);
        if let
            Some(r) =
            self.grid[column as usize]
                .iter()
                .position(|t| matches!(*t, TileState::Empty))
        {
            match team {
                Team::Cookie => self.grid[column as usize][r] = TileState::Cookie,
                Team::Milk => self.grid[column as usize][r] = TileState::Milk,
            }
            Ok(())
        } else {
            Err(From::from("Full column"))
        }
    }

    pub fn grid_update(&mut self) {
        self.raw_representation = format!(
            "⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜⬜⬜⬜⬜⬜\n",
            self.grid[0][3],self.grid[1][3],self.grid[2][3],self.grid[3][3],
            self.grid[0][2],self.grid[1][2],self.grid[2][2],self.grid[3][2],
            self.grid[0][1],self.grid[1][1],self.grid[2][1],self.grid[3][1],
            self.grid[0][0],self.grid[1][0],self.grid[2][0],self.grid[3][0],
        );
    }

    fn column_winner(&self) -> bool {
        self.grid
            .iter()
            .any(
                |column| {
                    column.iter().all(|t| matches!(t, TileState::Milk)) ||
                    column.iter().all(|t| matches!(t, TileState::Cookie))
                }
            )
    }

    fn row_winner(&self) -> bool {
        let mut rows: Vec<Vec<TileState>> = Vec::from([vec![], vec![], vec![], vec![]]);
        for c in self.grid {
            for (ri, r) in c.iter().enumerate() {
                rows[ri].push(*r);
            }
        }

        rows.iter().any(
                |row| {
                    row.iter().all(|t| matches!(t, TileState::Milk)) ||
                    row.iter().all(|t| matches!(t, TileState::Cookie))
                }
            )
    }

    fn diagonal_winner(&self) -> bool {
        let right_top_left_down = [
            self.grid[0][3], self.grid[1][2], self.grid[2][1], self.grid[3][0]
        ];

        if
            right_top_left_down.iter().all(|t| matches!(t, TileState::Milk)) ||
            right_top_left_down.iter().all(|t| matches!(t, TileState::Cookie))
        {
            return true;
        }

        let right_down_left_top = [
            self.grid[0][0], self.grid[1][1], self.grid[2][2], self.grid[3][3]
        ];

        right_down_left_top.iter().all(|t| matches!(t, TileState::Milk)) ||
        right_down_left_top.iter().all(|t| matches!(t, TileState::Cookie))
    }

    pub fn winner(&self) -> bool {
        if self.column_winner() == true {
            return true;
        }
        if self.row_winner() == true {
            return true;
        }
        if self.diagonal_winner() == true {
            return true;
        }
        false
    }

    pub fn full(&self) -> bool {
        !self.grid
            .iter()
            .any(
                |column| {
                    column.iter().any(|t| matches!(t, TileState::Empty))
                }
            )
    }
}

mod day_12 {
    use std::{mem, sync::Mutex};

    use actix_web::{ get, post, web, HttpResponse};
    use crate::challenges::day_12::{Board, Team};

    #[get("/board")]
    async fn board(board_state: web::Data<Mutex<Board>>) -> HttpResponse {
        let state = board_state.lock().unwrap();
        if state.full() {
            return HttpResponse::Ok().body(format!("{}No winner.\n", state.raw_representation));
        }

        let mut complement = String::new();
        if let Some(w) = &state.winner {
            complement = format!("{} wins!\n", w);
        }
        HttpResponse::Ok().body(format!("{}{}", state.raw_representation, complement))
    }

    #[post("/reset")]
    async fn reset(board_state: web::Data<Mutex<Board>>) -> HttpResponse {
        let mut state = board_state.lock().unwrap();
        let _ = mem::replace(&mut *state, Board::new());
        HttpResponse::Ok().body(state.raw_representation.clone())
    }

    #[post("/place/{team}/{column}")]
    async fn place(board_state: web::Data<Mutex<Board>>, path: web::Path<(String, String)>) -> HttpResponse {
        let (team, column) = path.into_inner();
        let team = match Team::try_from(team) {
            Ok(t) => t,
            Err(_) => return HttpResponse::BadRequest().finish()
        };

        let column = match column.parse::<u8>() {
            Err(_) => return HttpResponse::BadRequest().finish(),
            Ok(c) => {
                if c < 1 || c > 4 {
                    return HttpResponse::BadRequest().finish();
                }
                c
            }
        };
        let mut state = board_state.lock().unwrap();
        if let Some(w) = &state.winner {
            return HttpResponse::ServiceUnavailable().body(format!("{}{} wins!\n", state.raw_representation, w));
        }
        if state.full() {
            return HttpResponse::Ok().body(format!("{}No winner.\n", state.raw_representation));
        }

        if let Err(e) = state.set_position(&team, column.into()) {
            return HttpResponse::ServiceUnavailable().finish();
        }
        state.grid_update();

        let mut complement: String = String::new();
        if state.winner() == true {
            complement = format!("{} wins!\n", team);
            state.winner = Some(team);
        } else if state.full() {
            return HttpResponse::Ok().body(format!("{}No winner.\n", state.raw_representation));
        }

        HttpResponse::Ok().body(format!("{}{}", state.raw_representation, complement))
    }
}

pub fn scope() -> Scope {
    web::scope("/12")
        .service(day_12::board)
        .service(day_12::reset)
        .service(day_12::place)
}
