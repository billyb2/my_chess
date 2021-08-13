use std::ops::Range;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceType {
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
pub struct Piece {
    pub piece_type: PieceType,
    pub position: (u8, u8),
    pub black: bool,


}

pub fn check_movement(pieces: &[Piece], hovered_piece: &(Option<(u8, u8)>, Option<PieceType>), hovered_piece_pos: (u8, u8), selected_piece: &mut Option<(u8, u8)>, debug_text_to_draw: &mut String) -> (bool, bool) {
    let piece = pieces.iter().find(|piece| piece.position == selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();
    let piece_under_mouse = pieces.iter().find(|piece| piece.position == hovered_piece_pos && hovered_piece.1.unwrap() != PieceType::Dead);

    // First, check if the move is a move that this piece can usually make
    let (legal_move, can_kill) = match piece.piece_type {
        PieceType::Pawn => {
            let mut can_kill = false;

            // Basically, pawns can only move forward, or diagonally if you're killing an enemy piece
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
                        // Make sure that if the player is trying to move diagonally, they're only trying to move by 1 left or right
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

            let no_piece_between_horiz = |range_to_block_rook: Range<u8>| {
                pieces.iter().find(|other_piece|  {
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
        },
        PieceType::Bishop => {
            let up = hovered_piece_pos.1 < piece.position.1;

            let none_in_bishop_movement = |range_to_block_bishop: Vec<(u8, u8)>| -> bool {
                pieces.iter().find(|other_piece|  {                            
                    range_to_block_bishop.iter().find(|i| *i == &other_piece.position).is_some() && 
                    other_piece.position.0 != hovered_piece_pos.0 &&
                    other_piece.piece_type != PieceType::Dead
                }).is_none()
            };

            (match hovered_piece_pos.0 < piece.position.0 {
                // Left
                true => match up {
                    // Left-Up
                    true => {
                        // A diagonal range
                        let range_to_block_bishop = ((hovered_piece_pos.0..piece.position.0).into_iter().zip((hovered_piece_pos.1..piece.position.1).into_iter())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                        none_in_bishop_movement(range_to_block_bishop)

                        
                    },
                    // Left-Down
                    false => {
                        // A diagonal range
                        let range_to_block_bishop = ((hovered_piece_pos.0..piece.position.0).into_iter().zip((piece.position.1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                        none_in_bishop_movement(range_to_block_bishop)
                    
                    },
                },

                // Right
                false => match up {
                    // Right-Up
                    true => {
                        // A diagonal range
                        // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                        let range_to_block_bishop = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((hovered_piece_pos.1..piece.position.1).into_iter().rev())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                        *debug_text_to_draw = format!("{}, {:?}", piece.position.1, range_to_block_bishop);
                        
                        none_in_bishop_movement(range_to_block_bishop)
                    
                    },
                    
                    // Right-Down
                    false => {
                        // A diagonal range
                        // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                        let range_to_block_bishop = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((piece.position.1 + 1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y )).collect::<Vec<(u8, u8)>>();
                        *debug_text_to_draw = format!("{}, {:?}", piece.position.1, range_to_block_bishop);
                        
                        none_in_bishop_movement(range_to_block_bishop)
                    
                    },
                },

            }, true)
        
        },
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

}
