mod logic;

use macroquad::prelude::*;
use logic::*;

#[inline]
fn mouse_in_rectangle(coords: (f32, f32), size: (f32, f32)) -> bool {
    let mouse_pos = mouse_position();
    
    mouse_pos.0 > coords.0 && 
    mouse_pos.1 > coords.1 &&
    mouse_pos.0 < coords.0 + size.0 &&
    mouse_pos.1 < coords.1 + size.1 

}

// Some code I generated that contains the starting positions of all the pieces
const STARTING_PIECES: [Piece; 32] = [Piece { piece_type: PieceType::Pawn, position: (0, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (1, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (2, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (3, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (4, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (5, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (6, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (7, 1), black: true }, Piece { piece_type: PieceType::Pawn, position: (0, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (1, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (2, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (3, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (4, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (5, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (6, 6), black: false }, Piece { piece_type: PieceType::Pawn, position: (7, 6), black: false }, Piece { piece_type: PieceType::Rook, position: (0, 7), black: false }, Piece { piece_type: PieceType::Knight, position: (1, 7), black: false }, Piece { piece_type: PieceType::Bishop, position: (2, 7), black: false }, Piece { piece_type: PieceType::Queen, position: (4, 7), black: false }, Piece { piece_type: PieceType::King, position: (3, 7), black: false }, Piece { piece_type: PieceType::Bishop, position: (5, 7), black: false }, Piece { piece_type: PieceType::Knight, position: (6, 7), black: false }, Piece { piece_type: PieceType::Rook, position: (7, 7), black: false }, Piece { piece_type: PieceType::Rook, position: (0, 0), black: true }, Piece { piece_type: PieceType::Knight, position: (1, 0), black: true }, Piece { piece_type: PieceType::Bishop, position: (2, 0), black: true }, Piece { piece_type: PieceType::King, position: (3, 0), black: true }, Piece { piece_type: PieceType::Queen, position: (4, 0), black: true }, Piece { piece_type: PieceType::Bishop, position: (5, 0), black: true }, Piece { piece_type: PieceType::Knight, position: (6, 0), black: true }, Piece { piece_type: PieceType::Rook, position: (7, 0), black: true }];

#[macroquad::main("EmailChess")]
async fn main() {
    let mut pieces = STARTING_PIECES;
    let piece_size = screen_height() / 10.0;
    let mut selected_piece: Option<(u8, u8)> = None;

    #[cfg(debug_assertions)]
    let mut debug_text_to_draw = String::new();

    loop {
        clear_background(DARKGRAY);
        draw_text(&debug_text_to_draw, screen_width() / 2.0, 200.0, 17.0, BLACK);

        let mouse_down = is_mouse_button_pressed(MouseButton::Left);

        // Draws the board itself and the pieces on said board
        let hovered_piece = draw_board(piece_size, &pieces, selected_piece);

        if let Some(hovered_piece_pos) = hovered_piece.0 {
            #[cfg(debug_assertions)]
            {
                draw_text(&format!("{},{}", hovered_piece_pos.0, hovered_piece_pos.1), screen_width() / 2.0, 300.0, 50.0, BLACK);
                if let Some(p) = selected_piece {
                    draw_text(&format!("{},{}", p.0, p.1), screen_width() / 2.0, 250.0, 50.0, BLACK);

                }

            };
            
            // Moves and selects pieces
            if mouse_down {
                // Since a piece is already selected, we need to move the current piece
                // Don't let pieces move on top of other pieces, however

                if selected_piece.is_some() {
                    let (can_move, can_kill) = check_movement(&pieces, &hovered_piece, hovered_piece_pos, &mut selected_piece, &mut debug_text_to_draw);

                    if can_move {
                        if can_kill {
                            if let Some(piece_under_mouse) = pieces.iter_mut().find(|piece| piece.position == hovered_piece_pos) {
                                piece_under_mouse.piece_type = PieceType::Dead;

                            }

                        }


                        let piece = pieces.iter_mut().find(|piece| piece.position == selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();

                        piece.position = hovered_piece_pos;

                    }

                    // Whether the player moves or not, if they click after having a selected piece, then the selected piece shuold be reset 
                    selected_piece = None;

                // There is no piece selected, so it selects the piece the player is currently hovering over
                } else if let Some(piece) = pieces.iter_mut().find(|piece| piece.position == hovered_piece_pos && piece.piece_type != PieceType::Dead) {
                    selected_piece = match selected_piece.is_none() {
                        true => Some(piece.position),
                        false => None,
                    };

                }
            }
        }

        next_frame().await
    }

}

fn draw_board(piece_size: f32, pieces: &[Piece], selected_piece: Option<(u8, u8)>) -> (Option<(u8, u8)>, Option<PieceType>) {
    // Draws the actual board itself
    for x in 0_u8..8 {
        for y in 0_u8..8 {
            // The weird even odd stuff is for the alternating black and white checkerboard
            let black = match y.is_even() {
                true => x.is_odd(),
                false => x.is_even(),
            };

            let adj_x = x as f32 * piece_size;
            let adj_y = y as f32 * piece_size;
            
            // Check to see if the mouse is within the chess board
            // Have to do the second check since sometimes, the mouse pos is randomly at 0, 0
            let color = match selected_piece == Some((x, y)) {
                true => DARKGRAY,
                false => match mouse_in_rectangle((adj_x, adj_y), (piece_size, piece_size)) && mouse_position() != (0.0, 0.0) {
                    true => {
                        GRAY
                    },
                    false => match black {
                        true => BROWN,
                        false => BEIGE,
                    }
                }
            };

            draw_rectangle(adj_x, adj_y, piece_size, piece_size, color);

        }
    }


    // Draws all the pieces
    pieces.iter().for_each(|piece| {
        let piece_text = match piece.piece_type {
            PieceType::Pawn => "P",
            PieceType::Rook => "R",
            PieceType::Knight => "Kn",
            PieceType::Bishop => "B",
            PieceType::King => "Ki",
            PieceType::Queen => "Q",
            PieceType::Dead => "",
        };

        let adj_x = piece.position.0 as f32 * piece_size;
        let adj_y = piece.position.1 as f32 * piece_size;            
        
        // The text color should be the opposite of the board color
        draw_text(piece_text, adj_x + piece_size / 2.0 - 25.0, adj_y + piece_size / 2.0, 50.0, match piece.black {
            true => BLACK,
            false => WHITE,
        });

    });

    if mouse_in_rectangle((0.0, 0.0), (piece_size * 8.0, piece_size * 8.0)) {
        let x: u8 = (mouse_position().0 / piece_size).floor() as u8;
        let y: u8 = (mouse_position().1 / piece_size).floor() as u8;

        match pieces.iter().find(|p| p.position == (x, y)) {
            Some(piece) => (Some((x, y)), Some(piece.piece_type)),
            None => (Some((x, y)), None),
        }

    } else {
        (None, None)

    }

}
