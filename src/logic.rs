use std::ops::Range;
use std::convert::TryInto;

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

impl PieceType {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Pawn => "P",
            Self::Rook => "R",
            Self::Knight => "Kn",
            Self::Bishop => "B",
            Self::King => "Ki",
            Self::Queen => "Q",
            Self::Dead => "",
        }
    }

    pub fn to_bin(&self) -> u8 {
        match self {
            Self::Pawn => 1,
            Self::Rook => 2,
            Self::Knight => 3,
            Self::Bishop => 4,
            Self::King => 5,
            Self::Queen => 6,
            Self::Dead => 0,
        }
    }

    pub fn from_bin(bin: u8) -> Self {
        match bin {
            1 => Self::Pawn,
            2 => Self::Rook,
            3 => Self::Knight,
            4 => Self::Bishop,
            5 => Self::King,
            6 => Self::Queen,
            0 => Self::Dead,
            _ => unimplemented!(),
        }

    }

}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn to_bin(&self) -> u8 {
        match self {
            Self::Black => 0,
            Self::White => 1,
        }
    }

    pub fn from_bin(bin: u8) -> Self {
        match bin {
            0 => Self::Black,
            1 => Self::White,
            _ => unimplemented!(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub position: (u8, u8),
    pub color: PieceColor,
    // The number of moves that a piece has made (literally only for pawns)
    // Stored as a u32 since I don't want the game to crash if it goes on for a really long time
    pub num_of_moves: u32,

}

impl Piece {
    pub fn to_bin(&self) -> [u8; 8] {
        let num_mov_bin = self.num_of_moves.to_be_bytes();
        [self.piece_type.to_bin(), self.position.0, self.position.1, self.color.to_bin(), num_mov_bin[0], num_mov_bin[1], num_mov_bin[2], num_mov_bin[3]]
    }

    pub fn from_bin(bin: &[u8]) -> Self {
        Self {
            piece_type: PieceType::from_bin(bin[0]),
            position: (bin[1], bin[2]),
            color: PieceColor::from_bin(bin[3]),
            num_of_moves: u32::from_be_bytes(bin[4..].try_into().unwrap()),

        }
    }
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
    *debug_text_to_draw = format!("{} {}", hovered_piece_pos.0.distance(piece.position.0), (hovered_piece_pos.1.distance(piece.position.1)));

    // First, check if the move is a move that this piece can usually make
    let piece_move = match piece.piece_type {
        PieceType::Pawn => {
            let mut can_kill = false;

            // Basically, pawns can only move forward, or diagonally if you're killing an enemy piece
            // Make sure the pawn is only moving 1 space vertically, no matter what kind of move they're making
            let legal_move = match piece.color {
                // Black pawns can only move up
                PieceColor::Black => hovered_piece_pos.1 > piece.position.1 && match piece.num_of_moves {
                    // On their first move, pawns can move 1 or 2 spaces, but they can only move 2 if they're moving straight forward
                    0 =>  (hovered_piece_pos.1 - piece.position.1 == 2 && piece.position.0 == hovered_piece_pos.0) ||  hovered_piece_pos.1 - piece.position.1 == 1,
                    _ => hovered_piece_pos.1 - piece.position.1 == 1, 
                },
                // White pawns can only move down
                PieceColor::White => hovered_piece_pos.1 < piece.position.1 && match piece.num_of_moves {
                    // On their first move, pawns can move 1 or 2 spaces, but they can only move 2 if they're moving straight forward
                    0 =>  (piece.position.1 - hovered_piece_pos.1 == 2 && piece.position.0 == hovered_piece_pos.0) ||  piece.position.1 - hovered_piece_pos.1 == 1,
                    _ => piece.position.1 - hovered_piece_pos.1 == 1, 
                },
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

                                    no_piece_between_diag(range_to_block_diaganol)
                                
                                },
                                
                                // Right-Down
                                false => {
                                    // A diagonal range
                                    // For some reason backwards ranges don't work??? I need to just reverse the normal iterator for some reason
                                    let range_to_block_diaganol = ((piece.position.0 + 1..hovered_piece_pos.0).into_iter().zip((piece.position.1 + 1..hovered_piece_pos.1).into_iter())).map(|(x, y)| (x, y )).collect::<Vec<(u8, u8)>>();

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
            can_move: {
                ((hovered_piece_pos.0.distance(piece.position.0)) <= 1 ) && ((hovered_piece_pos.1.distance(piece.position.1)) <= 1)
            },
            can_kill: true,
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


    // Obviously, if it isn't a legal move, then don't let the player move at all
    match piece_move.can_move {
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
        // Checks the least significant bit for a 0, since if it's a zero, the integer is guaranteed to be even
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


pub trait ChessBoard {
    fn to_bin(&self) -> [u8; 256];
}

impl ChessBoard for [Piece; 32] {
    fn to_bin(&self) -> [u8; 256] {
        let piece_bytes: Vec<u8> = self.iter().flat_map(|piece| piece.to_bin()).collect();
        let map_bin: [u8; 256] = piece_bytes[..].try_into().unwrap();
        
        map_bin
    }
}

pub fn chess_board_from_bin(bin: [u8; 256]) -> [Piece; 32] {
    let bin_chunks = bin.chunks(8);
    bin_chunks.map(|chunk| Piece::from_bin(chunk)).collect::<Vec<Piece>>().as_slice().try_into().unwrap()

}