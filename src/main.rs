use std::ops::Range;
use macroquad::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    // Dead pieces exist so the pieces variable can be an array
    Dead,
}

#[derive(Copy, Clone, Debug)]
struct Piece {
    piece_type: PieceType,
    position: (u8, u8),
    black: bool,


}

#[inline]
fn mouse_in_rectangle(coords: (f32, f32), size: (f32, f32)) -> bool {
    let mouse_pos = mouse_position();
    
    mouse_pos.0 > coords.0 && 
    mouse_pos.1 > coords.1 &&
    mouse_pos.0 < coords.0 + size.0 &&
    mouse_pos.1 < coords.1 + size.1 

}

trait Evenness {
    fn is_even(&self) -> bool;
    fn is_odd(&self) -> bool;

}

impl Evenness for u8 {
    #[inline(always)]
    fn is_even(&self) -> bool {
        *self & 1 == 0
    }

    #[inline(always)]
    fn is_odd(&self) -> bool {
        !self.is_even()
    }
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
        draw_text(&debug_text_to_draw, screen_width() / 2.0, 200.0, 10.0, BLACK);

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
                    let (can_move, can_kill) = {
                        let piece = pieces.iter().find(|piece| piece.position == selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();
                        let piece_under_mouse = pieces.iter().find(|piece| piece.position == hovered_piece_pos && hovered_piece.1.unwrap() != PieceType::Dead);

                        // First, check if the move is a move that this piece can usually make
                        let (legal_move, can_kill) = match piece.piece_type {
                            PieceType::Pawn => {
                                let mut can_kill = false;

                                // Basically, pawns can only move forward, or diagnally if you're killing an enemy piece
                                // Make sure the pawn is only moving 1 space vertically, no matter what kind of move they're making
                                let legal_move = match piece.black {
                                    // White pawns can only move up
                                    true => hovered_piece_pos.1 > piece.position.1 && hovered_piece_pos.1 - piece.position.1 == 1,
                                    // Black pawns can only move down
                                    false => hovered_piece_pos.1 < piece.position.1 && piece.position.1 - hovered_piece_pos.1 == 1,
                                }
                                &&

                                (
                                    // Check that the pawn is moving straight forward
                                    piece.position.0 == hovered_piece_pos.0

                                    // If the player isn't moving straight forward, then do some checks
                                    || 
                                    (
                                        { 
                                            can_kill = true;

                                            // First, check that there is an enemy piece where the player is trying to move
                                            piece_under_mouse.is_some() && 
                                            // Make sure that if the player is trying to move diagnally, they're only trying to move by 1 left or right
                                            (piece_under_mouse.unwrap().position.1 + 1 == piece.position.1 || piece_under_mouse.unwrap().position.1 - 1 == piece.position.1)
                                            
                                        }
                                    )
                                );

                                (legal_move, can_kill)

                            },
                            PieceType::Rook => {
                                let no_piece_between_vert = |range_to_block_rook: Range<u8>| {
                                    pieces.iter().find(|other_piece|  {
                                        // Obviously, for a piece to block the rook, it needs to be on the same X axis (when moving vertically)
                                        other_piece.position.0 == piece.position.0 && 
                                        // Then, there can be no pieces in between where the player is trying to move and where it's moving
                                        range_to_block_rook.contains(&other_piece.position.1) && 
                                        // If the other piece is at the very exact position the rook was trying to move, it will kill it
                                        other_piece.position.1 != hovered_piece_pos.1 &&
                                        // Obviously, the piece can't be dead
                                        other_piece.piece_type != PieceType::Dead
                                    }).is_none()
                                };

                                let mut no_piece_between_horiz = |range_to_block_rook: Range<u8>| {
                                    pieces.iter().find(|other_piece|  {
                                        debug_text_to_draw = format!("{:?}", other_piece);
                                        // Obviously, for a piece to block the rook, it needs to be on the same Y axis (when moving vertically)
                                        other_piece.position.1 == piece.position.1 && 
                                        // Then, there can be no pieces in between where the player is trying to move and where it's moving
                                        range_to_block_rook.contains(&other_piece.position.0) && 
                                        // If the other piece is at the very exact position the rook was trying to move, it will kill it
                                        other_piece.position.0 != hovered_piece_pos.0 &&
                                        // Obviously, the piece can't be dead
                                        other_piece.piece_type != PieceType::Dead
                                    }).is_none()
                                };


                                ((piece.position.0 == hovered_piece_pos.0 &&
                                    // Check to see if the rook is moving up or down
                                     match hovered_piece_pos.1 < piece.position.1 {
                                         // Moving up
                                        true => {
                                            let range_to_block_rook = hovered_piece_pos.1..piece.position.1;
                                            // There can't be any pieces between the place where the rook is, and where it's attempting to move
                                            no_piece_between_vert(range_to_block_rook)

                                        },
                                        // Moving down
                                        false => {
                                            let range_to_block_rook = piece.position.1 + 1..hovered_piece_pos.1;
                                            no_piece_between_vert(range_to_block_rook)
                                            
                                        },

                                }) 
                                // Rooks can move vertically or horizontally, but not both, hence the XOR
                                ^
                                 (piece.position.1 == hovered_piece_pos.1 && match hovered_piece_pos.0 < piece.position.0 {
                                    // Left
                                    true => {
                                        let range_to_block_rook = hovered_piece_pos.0..piece.position.0;
                                        // There can't be any pieces between the place where the rook is, and where it's attempting to move
                                        no_piece_between_horiz(range_to_block_rook)

                                    },
                                    // Right
                                    false => {
                                        let range_to_block_rook = piece.position.0 + 1..hovered_piece_pos.0;
                                        no_piece_between_horiz(range_to_block_rook)
                                        
                                    },

                                 }), true)
                            }
                            _ => (false, false),

                        };


                        match legal_move {
                            // Obviously, if it isn't a legal move, then don't let the player move at all
                            true => match piece_under_mouse {
                                // If it is a legal move, only move onto a piece if it's an enemy piece (piece.black ^ piece_under_mouse.black)
                                // Also only move onto that piece if it's able to kill it
                                Some(piece_under_mouse) => (piece.black ^ piece_under_mouse.black && can_kill, can_kill),
                                None => (true, false)
                            }
                            false => (false, false)

                        }


                    };

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
