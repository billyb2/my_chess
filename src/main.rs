use std::convert::TryInto;
use macroquad::prelude::*;

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone)]
struct Piece {
    piece_type: PieceType,
    position: (u8, u8),
    black: bool,


}

impl Piece {
    fn new(piece_type: PieceType, position: (u8, u8), black: bool) -> Self {
        Piece {
           piece_type,
           position,
           black,
            
        }

    }

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

#[macroquad::main("EmailChess")]
async fn main() {
    let starting_pieces: [Piece; 32] = {
        let mut pieces: [Piece; 32] = [Piece::new(PieceType::Pawn, (0, 0), true); 32];

        let specials = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::King, PieceType::Queen, PieceType::Bishop,PieceType::Knight, PieceType::Rook];

        // Black pawns
        for (i, piece) in pieces[0..8].iter_mut().enumerate() {
            piece.position = (i.try_into().unwrap(), 1);
    
        }
    
        // White pawns
        for (i, piece) in pieces[8..16].iter_mut().enumerate() {
            piece.position = (i.try_into().unwrap(), 6);
            piece.black = false;
    
        }

        // White special pieces
        for (i, (piece, special_piece)) in pieces[16..24].iter_mut().zip(specials.iter()).enumerate() {
            piece.position = (i.try_into().unwrap(), 7);
            piece.black = false;
            piece.piece_type = *special_piece;

        }

        // Black special pieces
        for (i, (piece, special_piece)) in pieces[24..32].iter_mut().zip(specials.iter()).enumerate() {
            piece.position = (i.try_into().unwrap(), 0);
            piece.piece_type = *special_piece;

        }
    
        pieces
    };

    let mut pieces = starting_pieces;
    let piece_size = screen_height() / 10.0;
    let mut selected_piece: Option<(u8, u8)> = None;

    loop {
        clear_background(DARKGRAY);
        draw_text("CHESS", screen_width() / 2.0, 200.0, 50.0, BLACK);

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
                    let (can_move, should_kill) = {
                        let piece = pieces.iter().find(|piece| piece.position == selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();
                        let piece_under_mouse = pieces.iter().find(|piece| piece.position == hovered_piece_pos && hovered_piece.1.unwrap() != PieceType::Dead);

                        let mut moved_diagnally = false;

                        // First, check if the move is a move that this piece can usually make
                        let legal_move = match piece.piece_type {
                            PieceType::Pawn => {
                                // Basically, pawns can only move forward, or diagnally if you're killing an enemy piece
                                // Make sure the pawn is only moving 1 space vertically, no matter what kind of move they're making
                                (
                                    match piece.black {
                                        // White pawns can only move up
                                        true => hovered_piece_pos.1 > piece.position.1 && hovered_piece_pos.1 - piece.position.1 == 1,
                                        // Black pawns can only move down
                                        false => hovered_piece_pos.1 < piece.position.1 && piece.position.1 - hovered_piece_pos.1 == 1,
                                    }
                                ) &&

                                (
                                    // Check that the pawn is moving straight forward
                                    piece.position.0 == hovered_piece_pos.0

                                    // If the player isn't moving straight forward, then do some checks
                                    || 
                                    (
                                        {
                                            moved_diagnally = 
                                            // First, check that there is an enemy piece where the player is trying to move
                                            piece_under_mouse.is_some() && 
                                            // Make sure that if the player is trying to move diagnally, they're only trying to move by 1 left or right
                                            (piece_under_mouse.unwrap().position.0 + 1 == piece.position.0 || piece_under_mouse.unwrap().position.0 - 1 == piece.position.0);


                                            moved_diagnally
                                        }
                                    )
                                )

                            },
                            _ => false,

                        };


                        match legal_move {
                            true => match piece_under_mouse {
                                // One of the pieces can be black, but not both for it to be able to move
                                Some(piece_under_mouse) => (piece.black ^ piece_under_mouse.black && moved_diagnally, moved_diagnally),
                                None => (true, false)
                            }
                            false => (false, false)

                        }


                    };

                    if can_move {
                        if should_kill {
                            if let Some(piece_under_mouse) = pieces.iter_mut().find(|piece| piece.position == hovered_piece_pos) {
                                piece_under_mouse.piece_type = PieceType::Dead;

                            }
                            

                        }


                        let piece = pieces.iter_mut().find(|piece| piece.position == selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();

                        piece.position = hovered_piece_pos;
                        selected_piece = None;

                    }

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
