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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceColor {
    Black,
    White,
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub position: (u8, u8),
    pub color: PieceColor,


}

pub struct Move {
    pub can_move: bool,
    pub can_kill: bool,
}

// The check movement function returns 
pub fn check_movement(pieces: &[Piece], hovered_piece: &(Option<(u8, u8)>, Option<PieceType>), hovered_piece_pos: (u8, u8), selected_piece: &mut Option<(u8, u8)>, debug_text_to_draw: &mut String) -> Move {
    let piece = pieces.iter().find(|piece| piece.position == selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();
    let piece_under_mouse = pieces.iter().find(|piece| piece.position == hovered_piece_pos && hovered_piece.1.unwrap() != PieceType::Dead);

    let up = hovered_piece_pos.1 < piece.position.1;

    let no_piece_between_diag = |range_to_block_diaganol: Vec<(u8, u8)>| -> bool {
        !pieces.iter().any(|other_piece|  {                            
            range_to_block_diaganol.iter().any(|i| *i == other_piece.position) && 
            other_piece.position.0 != hovered_piece_pos.0 &&
            other_piece.piece_type != PieceType::Dead
        })
    };

    let no_piece_between_vert = |range_to_block_horiz: Range<u8>| {
        !pieces.iter().any(|other_piece|  {
            // Obviously, for a piece to block the rook, it needs to be on the same X axis (when moving vertically)
            other_piece.position.0 == piece.position.0 && 
            // Then, there can be no pieces in between where the player is trying to move and where it's moving
            range_to_block_horiz.contains(&other_piece.position.1) && 
            // If the other piece is at the very exact position the rook was trying to move, it will kill it
            other_piece.position.1 != hovered_piece_pos.1 &&
            // Obviously, the piece can't be dead
            other_piece.piece_type != PieceType::Dead
        })
    };

    let no_piece_between_horiz = |range_to_block_horiz: Range<u8>| {
        !pieces.iter().any(|other_piece|  {
            // Obviously, for a piece to block the rook, it needs to be on the same Y axis (when moving vertically)
            other_piece.position.1 == piece.position.1 && 
            // Then, there can be no pieces in between where the player is trying to move and where it's moving
            range_to_block_horiz.contains(&other_piece.position.0) && 
            // If the other piece is at the very exact position the rook was trying to move, it will kill it
            other_piece.position.0 != hovered_piece_pos.0 &&
            // Obviously, the piece can't be dead
            other_piece.piece_type != PieceType::Dead
        })
    };


    // First, check if the move is a move that this piece can usually make
    let piece_move = match piece.piece_type {
        PieceType::Pawn => {
            let mut can_kill = false;

            // Basically, pawns can only move forward, or diagonally if you're killing an enemy piece
            // Make sure the pawn is only moving 1 space vertically, no matter what kind of move they're making
            let legal_move = match piece.color {
                // Black pawns can only move up
                PieceColor::Black => hovered_piece_pos.1 > piece.position.1 && hovered_piece_pos.1 - piece.position.1 == 1,
                // White pawns can only move down
                PieceColor::White => hovered_piece_pos.1 < piece.position.1 && piece.position.1 - hovered_piece_pos.1 == 1,
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

            Move {
                can_move: legal_move,
                can_kill,
            }

        },
        PieceType::Rook => {
            Move {
                can_move: (piece.position.0 == hovered_piece_pos.0 &&
                // Check to see if the rook is moving up or down
                 match hovered_piece_pos.1 < piece.position.1 {
                     // Moving up
                    true => {
                        let range_to_block_horiz = hovered_piece_pos.1..piece.position.1;
                        // There can't be any pieces between the place where the rook is, and where it's attempting to move
                        no_piece_between_vert(range_to_block_horiz)

                    },
                    // Moving down
                    false => {
                        let range_to_block_horiz = piece.position.1 + 1..hovered_piece_pos.1;
                        no_piece_between_vert(range_to_block_horiz)
                        
                    },

            }) 
            // Rooks can move vertically or horizontally, but not both, hence the XOR
            ^
             (piece.position.1 == hovered_piece_pos.1 && match hovered_piece_pos.0 < piece.position.0 {
                // Left
                true => {
                    let range_to_block_horiz = hovered_piece_pos.0..piece.position.0;
                    // There can't be any pieces between the place where the rook is, and where it's attempting to move
                    no_piece_between_horiz(range_to_block_horiz)

                },
                // Right
                false => {
                    let range_to_block_horiz = piece.position.0 + 1..hovered_piece_pos.0;
                    no_piece_between_horiz(range_to_block_horiz)
                    
                },

             }),
             // Rooks can always kill
             can_kill: true,
            }
        },
        PieceType::Bishop => { 
            Move {
                can_move: hovered_piece_pos.0.distance(piece.position.0) == hovered_piece_pos.1.distance(piece.position.1) &&
                match hovered_piece_pos.0 < piece.position.0 {
                // Left
                true => match up {
                    // Left-Up
                    true => {
                        // A diagonal range
                        let range_to_block_diaganol = ((hovered_piece_pos.0..piece.position.0).into_iter().zip((hovered_piece_pos.1..piece.position.1).into_iter())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                        no_piece_between_diag(range_to_block_diaganol)

                        
                    },
                    // Left-Down
                    false => {
                        // A diagonal range
                        let range_to_block_diaganol = ((hovered_piece_pos.0..piece.position.0).into_iter().zip((piece.position.1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                        no_piece_between_diag(range_to_block_diaganol)
                    
                    },
                },

                // Right
                false => match up {
                    // Right-Up
                    true => {
                        // A diagonal range
                        // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                        let range_to_block_diaganol = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((hovered_piece_pos.1..piece.position.1).into_iter().rev())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                        *debug_text_to_draw = format!("{}, {:?}", piece.position.1, range_to_block_diaganol);
                        
                        no_piece_between_diag(range_to_block_diaganol)
                    
                    },
                    
                    // Right-Down
                    false => {
                        // A diagonal range
                        // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                        let range_to_block_diaganol = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((piece.position.1 + 1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y )).collect::<Vec<(u8, u8)>>();
                        *debug_text_to_draw = format!("{}, {:?}", piece.position.1, range_to_block_diaganol);
                        
                        no_piece_between_diag(range_to_block_diaganol)
                    
                    },
                },

                },
                // Bishops can always kill
                can_kill: true,
            }
        
        },
        PieceType::Queen => {
            Move {
                can_move: 
                    match piece.position.0.distance(hovered_piece_pos.0) == piece.position.1.distance(hovered_piece_pos.1) {
                        // If that's true, then the player is moving diagnally, and should use bishop rules
                        true => (
                            hovered_piece_pos.0.distance(piece.position.0) == hovered_piece_pos.1.distance(piece.position.1) &&
                            match hovered_piece_pos.0 < piece.position.0 {
                            // Left
                            true => match up {
                                // Left-Up
                                true => {
                                    // A diagonal range
                                    let range_to_block_diaganol = ((hovered_piece_pos.0..piece.position.0).into_iter().zip((hovered_piece_pos.1..piece.position.1).into_iter())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                                    no_piece_between_diag(range_to_block_diaganol)
            
                                    
                                },
                                // Left-Down
                                false => {
                                    // A diagonal range
                                    let range_to_block_diaganol = ((hovered_piece_pos.0..piece.position.0).into_iter().zip((piece.position.1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                                    no_piece_between_diag(range_to_block_diaganol)
                                
                                },
                            },
            
                            // Right
                            false => match up {
                                // Right-Up
                                true => {
                                    // A diagonal range
                                    // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                                    let range_to_block_diaganol = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((hovered_piece_pos.1..piece.position.1).into_iter().rev())).map(|(x, y)| (x, y)).collect::<Vec<(u8, u8)>>();
                                    *debug_text_to_draw = format!("{}, {:?}", piece.position.1, range_to_block_diaganol);
                                    
                                    no_piece_between_diag(range_to_block_diaganol)
                                
                                },
                                
                                // Right-Down
                                false => {
                                    // A diagonal range
                                    // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                                    let range_to_block_diaganol = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((piece.position.1 + 1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y )).collect::<Vec<(u8, u8)>>();
                                    *debug_text_to_draw = format!("{}, {:?}", piece.position.1, range_to_block_diaganol);
                                    
                                    no_piece_between_diag(range_to_block_diaganol)
                                
                                },
                            },
            
                        }),
                        // If this is false, then we need to check if it's moving like a rook
                        false => (piece.position.0 == hovered_piece_pos.0 &&
                            // Check to see if the rook is moving up or down
                            match hovered_piece_pos.1 < piece.position.1 {
                                // Moving up
                                true => {
                                    let range_to_block_horiz = hovered_piece_pos.1..piece.position.1;
                                    // There can't be any pieces between the place where the rook is, and where it's attempting to move
                                    no_piece_between_vert(range_to_block_horiz)
            
                                },
                                // Moving down
                                false => {
                                    let range_to_block_horiz = piece.position.1 + 1..hovered_piece_pos.1;
                                    no_piece_between_vert(range_to_block_horiz)
                                    
                                },
            
                        }) 
                        // Rooks can move vertically or horizontally, but not both, hence the XOR
                        ^
                        (piece.position.1 == hovered_piece_pos.1 && match hovered_piece_pos.0 < piece.position.0 {
                            // Left
                            true => {
                                let range_to_block_horiz = hovered_piece_pos.0..piece.position.0;
                                // There can't be any pieces between the place where the rook is, and where it's attempting to move
                                no_piece_between_horiz(range_to_block_horiz)
            
                            },
                            // Right
                            false => {
                                let range_to_block_horiz = piece.position.0 + 1..hovered_piece_pos.0;
                                no_piece_between_horiz(range_to_block_horiz)
                                
                            },
            
                        }),
                    }, 
                can_kill: true,
            }
            

        },
        //TODO: Add king
        PieceType::King => Move {
            can_move: false,
            can_kill: false,
        },
        PieceType::Knight => Move {
            can_move: {
                // Just generates all the possible L shapes
                // All the subtractions are purposely wrapping to prevent crashes on underflows, since it's honestly fine if the subtractions underflows since it's impossible to select a board position above 7
                // There's also no way for the addition to overflow (since the max it will ever go to is 8), so we don't need a check for that 
                let possible_moves: [(u8, u8); 8] = [
                    (piece.position.0 + 1, piece.position.1.wrapping_sub(2)),
                    (piece.position.0.wrapping_sub(1), piece.position.1.wrapping_sub(2)),

                    (piece.position.0 + 1, piece.position.1 + 2),
                    (piece.position.0.wrapping_sub(1), piece.position.1 + 2),

                    (piece.position.0 + 2, piece.position.1 + 1),
                    (piece.position.0 + 2, piece.position.1.wrapping_sub(1)),
                    
                    (piece.position.0.wrapping_sub(2), piece.position.1 + 1),
                    (piece.position.0.wrapping_sub(2), piece.position.1.wrapping_sub(1))

                ];

                // Just check that the player's move is one of 8 possible knight moves
                possible_moves.contains(&hovered_piece_pos)

            },
            can_kill: true,
        },
        // Dead pieces will never be able to kill or move
        PieceType::Dead => Move {
            can_move: false,
            can_kill: false,
        }

    };


    match piece_move.can_move {
        // Obviously, if it isn't a legal move, then don't let the player move at all
        true => match piece_under_mouse {
            // If it is a legal move, only move onto a piece if it's an enemy piece (piece.color != piece_under_mouse.color)
            // Also only move onto that piece if it's able to kill it
            Some(piece_under_mouse) => Move {
                can_move: piece.color != piece_under_mouse.color && piece_move.can_kill,
                can_kill: piece_move.can_kill,
            },
            None => Move {
                can_move: true,
                can_kill: false,
            }
        }
        false => Move {
            can_move: false,
            can_kill: false,
        }

    }

}

// A trait I made for integers jut to make my life easer
pub trait MyNumTrait {
    fn is_even(&self) -> bool;
    fn is_odd(&self) -> bool;
    fn distance(self, y: u8) -> u8;

}

impl MyNumTrait for u8 {
    #[inline(always)]
    fn is_even(&self) -> bool {
        *self & 1 == 0
    }

    #[inline(always)]
    fn is_odd(&self) -> bool {
        !self.is_even()
    }

    #[inline(always)]
    fn distance(self, y: u8) -> u8 {
        match self < y {
            true => y - self,
            false => self - y,
        }
    }
}
