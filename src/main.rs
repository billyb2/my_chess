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
    color: Color,

}

impl Piece {
    fn new(piece_type: PieceType, position: (u8, u8), color: Color) -> Self {
        Piece {
           piece_type,
           position,
           color,
            
        }

    }

}

fn mouse_in_rectangle(coords: (f32, f32), size: (f32, f32)) -> bool {
    let mouse_pos = mouse_position();
    
    mouse_pos.0 > coords.0 && 
    mouse_pos.1 > coords.1 &&
    mouse_pos.0 < coords.0 + size.0 &&
    mouse_pos.1 < coords.1 + size.1 

}

trait Evenness {
    fn is_even(self) -> bool;
    fn is_odd(self) -> bool;

}

impl Evenness for u8 {
    #[inline(always)]
    fn is_even(self) -> bool {
        self & 1 == 0
    }

    #[inline(always)]
    fn is_odd(self) -> bool {
        !self.is_even()
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let starting_pieces: [Piece; 32] = {
        let mut pieces: [Piece; 32] = [Piece::new(PieceType::Pawn, (0, 0), BLACK); 32];
        //let specials = [Piece::new(PieceType::Rook, (0, 0), BLACK), Piece::new(PieceType::Knight, (0, 0), BLACK), Piece::new(PieceType::Bishop, (0, 0), BLACK)];
        let specials = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::King, PieceType::Queen, PieceType::Bishop,PieceType::Knight, PieceType::Rook];

        // Black pawns
        for (i, piece) in pieces[0..8].iter_mut().enumerate() {
            piece.position = (i.try_into().unwrap(), 1);
    
        }
    
        // White pawns
        for (i, piece) in pieces[8..16].iter_mut().enumerate() {
            piece.position = (i.try_into().unwrap(), 6);
            piece.color = WHITE;
    
        }

        // White special pieces
        for (i, (piece, special_piece)) in pieces[16..24].iter_mut().zip(specials.iter()).enumerate() {
            piece.position = (i.try_into().unwrap(), 7);
            piece.color = WHITE;
            piece.piece_type = *special_piece;
        }

        // Black special pieces
        for (i, (piece, special_piece)) in pieces[24..32].iter_mut().zip(specials.iter()).enumerate() {
            piece.position = (i.try_into().unwrap(), 0);
            piece.color = BLACK;
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
                    draw_text(&format!("{}", pieces.iter().filter(|piece| piece.position == hovered_piece_pos).count()), screen_width() / 2.0, 150.0, 50.0, BLACK);

                }

            };
            
            // Moves and selects pieces
            if mouse_down {
                // Since a piece is already selected, we need to move the current piece
                // Don't let pieces move on top of other pieces, however
                if selected_piece.is_some() && pieces.iter().filter(|piece| piece.position == hovered_piece_pos).count() == 0 {
                    let piece = pieces.iter_mut().find(|piece| piece.position == selected_piece.unwrap()).unwrap();

                    piece.position = hovered_piece_pos;
                    selected_piece = None;

                // There is no piece selected, so it selects the one the player is currently hovering over
                } else {
                    if let Some(piece) = pieces.iter_mut().find(|piece| piece.position == hovered_piece_pos) {
                        selected_piece = Some(piece.position);

                    }

                }
            }
        }

        next_frame().await
    }


    // Move the piece
    /*if mouse_down && *piece != Piece::Empty {
        if let Some(sel_piece) = selected_piece {
            board[sel_piece.1][sel_piece.0] = *piece;
            selected_piece = None;
        }
    }*/

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
        draw_text(piece_text, adj_x + piece_size / 2.0 - 25.0, adj_y + piece_size / 2.0, 50.0, piece.color);

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

            // Depending on the position of the piece, the piece un
            /*let black = match y.is_even() {
                true => x.is_odd(),
                false => x.is_even(),
            };
            
            // Adjust the coordinates to the desired piece size
            let adj_x = x as f32 * piece_size;
            let adj_y = y as f32 * piece_size;

            #[cfg(debug_assertions)]
            if let Some(p) = selected_piece {
                draw_text(&p.0.to_string(), 800.0, 200.0, 25.0, WHITE);
                draw_text(&p.1.to_string(), 825.0, 200.0, 25.0, WHITE);
            }

            // Move the piece
            if mouse_down && *piece != Piece::Empty && selected_piece.is_none() {
                selected_piece = Some((x, y));
            }

            // If the mouse is in the rectangle, the color is gray. If the piece is selected, the color is gray If not, then the color alternates
            let color = match selected_piece == Some((x, y)) {
                true => DARKGRAY,
                false => match mouse_in_rectangle((adj_x, adj_y), (piece_size, piece_size)) {
                    false => match black {
                         true => BLACK,
                         false => WHITE,
                     },
                    true => GRAY,
                 }
            };

            draw_rectangle(adj_x, adj_y, piece_size, piece_size, color);

            let piece_text = match piece.piece_type {
                Piece::Pawn => "P",
                Piece::Rook => "R",
                Piece::Knight => "Kn",
                Piece::Bishop => "B",
                Piece::King => "Ki",
                Piece::Queen => "Q",
            };
            
            // The text color should be the opposite of the board color
            draw_text(piece_text, adj_x + piece_size / 2.0 - 25.0, adj_y + piece_size / 2.0, 50.0, match black {
                true => WHITE,
                false => BLACK,
            });*/
