mod logic;
mod chess;

use macroquad::prelude::*;
use logic::*;
use chess::ChessGame;

// Some code I generated that contains the starting positions of all the pieces
#[macroquad::main(window_conf)]
async fn main() {
    let mut stage: Box<dyn GameStage> = Box::new(ChessGame::new());

    loop {
        // Run the game logic of the stage, then do any drawing
        stage.logic();
        stage.draw();
        // If the stage is going to change, then it needs to tell the game loop
        if let Some(new_stage) = stage.set_new_stage() {
            stage = match new_stage {
                Stages::ChessGame => Box::new(ChessGame::new()),
            }

        }

        next_frame().await

    }

}

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Chess".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}