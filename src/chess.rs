use std::ops::Range;
use std::convert::TryInto;
use std::io::Cursor;

#[cfg(target_arch = "wasm32")]
use sapp_jsutils::JsObject;
use base64::{encode_config, decode_config};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use brotli::{BrotliCompress, BrotliDecompress};
use brotli::enc::BrotliEncoderParams;

use macroquad::prelude::*;

use crate::logic::*;

const STARTING_PIECES: [Piece; 32] = [Piece { piece_type: PieceType::Pawn, position: (0, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (1, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (2, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (3, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (4, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (5, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (6, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (7, 1), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Pawn, position: (0, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (1, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (2, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (3, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (4, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (5, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (6, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Pawn, position: (7, 6), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Rook, position: (0, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Knight, position: (1, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Bishop, position: (2, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Queen, position: (4, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::King, position: (3, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Bishop, position: (5, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Knight, position: (6, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Rook, position: (7, 7), num_of_moves: 0, color: PieceColor::White }, Piece { piece_type: PieceType::Rook, position: (0, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Knight, position: (1, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Bishop, position: (2, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::King, position: (3, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Queen, position: (4, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Bishop, position: (5, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Knight, position: (6, 0), num_of_moves: 0, color: PieceColor::Black }, Piece { piece_type: PieceType::Rook, position: (7, 0), num_of_moves: 0, color: PieceColor::Black }];

pub struct ChessGame {
    pub pieces: [Piece; 32],
    pub piece_size: f32,
    pub selected_piece: Option<(u8, u8)>,
    pub white_turn: bool,

}

impl ChessGame {
    fn draw_board(&self) {
        // Draws the actual board itself
        for x in 0_u8..8 {
            for y in 0_u8..8 {
                // The weird even odd stuff is for the alternating black and white checkerboard
                let black = match y.is_even() {
                    true => x.is_odd(),
                    false => x.is_even(),
                };
    
                let adj_x = x as f32 * self.piece_size;
                let adj_y = y as f32 * self.piece_size;
                
                // Check to see if the mouse is within the chess board
                // Have to do the second check since sometimes, the mouse pos is randomly at 0, 0
                let color = match self.selected_piece == Some((x, y)) {
                    true => DARKGRAY,
                    false => match mouse_in_rectangle((adj_x, adj_y), (self.piece_size, self.piece_size)) && mouse_position() != (0.0, 0.0) {
                        true => {
                            GRAY
                        },
                        false => match black {
                            true => BROWN,
                            false => BEIGE,
                        }
                    }
                };
    
                draw_rectangle(adj_x, adj_y, self.piece_size, self.piece_size, color);
    
            }
        }
    
    
        // Draws all the pieces
        self.pieces.iter().for_each(|piece| {
            let piece_text = piece.piece_type.to_str();
    
            let adj_x = piece.position.0 as f32 * self.piece_size;
            let adj_y = piece.position.1 as f32 * self.piece_size;            
            
            // The text PieceColor should be the opposite of the board PieceColor
            draw_text(piece_text, adj_x + self.piece_size / 2.0 - 25.0, adj_y + self.piece_size / 2.0, 50.0, match piece.color {
                PieceColor::Black => BLACK,
                PieceColor::White => WHITE,
            });
    
        });
    
    }
    
    fn get_hovered_piece(&mut self) -> (Option<(u8, u8)>, Option<PieceType>) {
        if mouse_in_rectangle((0.0, 0.0), (self.piece_size * 8.0, self.piece_size * 8.0)) {
            let x: u8 = (mouse_position().0 / self.piece_size).floor() as u8;
            let y: u8 = (mouse_position().1 / self.piece_size).floor() as u8;
    
            match self.pieces.iter().find(|p| p.position == (x, y)) {
                Some(piece) => (Some((x, y)), Some(piece.piece_type)),
                None => (Some((x, y)), None),
            }
    
        } else {
            (None, None)
    
        }
    }

    pub fn new() -> Self {
        Self {
            pieces: STARTING_PIECES,
            piece_size: (screen_width() + screen_height()) / 30.0,
            selected_piece: None,
            white_turn: true,

        }
    }

    // The check movement function returns 
    fn check_movement(&mut self, hovered_piece: &(Option<(u8, u8)>, Option<PieceType>), hovered_piece_pos: (u8, u8)) -> Move {
        let piece = self.pieces.iter().find(|piece| piece.position == self.selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap();
        let piece_under_mouse = self.pieces.iter().find(|piece| piece.position == hovered_piece_pos && hovered_piece.1.unwrap() != PieceType::Dead);

        let up = hovered_piece_pos.1 < piece.position.1;

        let no_piece_between_diag = |range_to_block_diaganol: Vec<(u8, u8)>| -> bool {
            !self.pieces.iter().any(|other_piece|  {                            
                range_to_block_diaganol.iter().any(|i| *i == other_piece.position) && 
                other_piece.position.0 != hovered_piece_pos.0 &&
                other_piece.piece_type != PieceType::Dead
            })
        };

        let no_piece_between_vert = |range_to_block_horiz: Range<u8>| {
            !self.pieces.iter().any(|other_piece|  {
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
            !self.pieces.iter().any(|other_piece|  {
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
        
}

impl GameStage for ChessGame {
    fn draw(&self) {
        clear_background(DARKGRAY);
        self.draw_board();


    }

    fn logic(&mut self) {
        let hovered_piece = self.get_hovered_piece();
        let mouse_down = is_mouse_button_pressed(MouseButton::Left);


        if let Some(hovered_piece_pos) = hovered_piece.0 {
            draw_text(&format!("{},{}", hovered_piece_pos.0, hovered_piece_pos.1), screen_width() / 2.0, 300.0, 50.0, BLACK);
            if let Some(p) = self.selected_piece {
                draw_text(&format!("{},{}", p.0, p.1), screen_width() / 2.0, 250.0, 50.0, BLACK);

            }
        
            // Moves and selects pieces
            if mouse_down {
                // Since a piece is already selected, we need to move the current piece
                // Don't let pieces move on top of other pieces, however

                if self.selected_piece.is_some() {
                    let piece_move = self.check_movement(&hovered_piece, hovered_piece_pos);

                    if piece_move.can_move {
                        let piece_is_white = self.pieces.iter().find(|piece| piece.position == self.selected_piece.unwrap() && piece.piece_type != PieceType::Dead).unwrap().color == PieceColor::White;

                        if (piece_is_white && self.white_turn) || (!piece_is_white && !self.white_turn) {
                            if piece_move.can_kill {
                                if let Some(piece_under_mouse) = self.pieces.iter_mut().find(|piece| piece.position == hovered_piece_pos) {
                                    piece_under_mouse.piece_type = PieceType::Dead;

                                }

                            }


                            let sel_piece = self.selected_piece.unwrap();
                            let piece = self.pieces.iter_mut().find(|piece| piece.position == sel_piece && piece.piece_type != PieceType::Dead).unwrap();

                            piece.position = hovered_piece_pos;
                            piece.num_of_moves += 1;
                            
                            self.white_turn = !self.white_turn;

                            #[cfg(target_arch = "wasm32")]
                            {
                                let mut board_bin = Cursor::new(self.pieces.to_bin());
                                let mut compressed_board_bin = Cursor::new(Vec::with_capacity(100));
                                let params = BrotliEncoderParams::default();
                                BrotliCompress(&mut board_bin, &mut compressed_board_bin, &params).unwrap();
                                //let compressed_board_bin = compress_prepend_size(&board_bin);

                                let board_bin_as_ascii85 = JsObject::string(&encode_config(&compressed_board_bin.into_inner(), base64::URL_SAFE_NO_PAD));

                                // Tests the to_bin and from_bin functions
                                #[cfg(debug_assertions)]
                                {
                                    let mut board_string = String::with_capacity(200);

                                    JsObject::to_string(&board_bin_as_ascii85, &mut board_string);

                                    let mut compressed_board_bin = Cursor::new(decode_config(&board_string, base64::URL_SAFE_NO_PAD).unwrap());
                                    let mut board_bin = Cursor::new(Vec::with_capacity(256));
                                    BrotliDecompress(&mut compressed_board_bin, &mut board_bin).unwrap();
                                    //let board_bin = decompress_size_prepended(&compressed_board_bin).unwrap();

                                    let board = chess_board_from_bin(board_bin.into_inner().try_into().unwrap());
                                    assert_eq!(board, self.pieces);

                                }
                            
                                unsafe { send_board(board_bin_as_ascii85) };
                            }

                        }

                    }

                    // Whether the player moves or not, if they click after having a selected piece, then the selected piece shuold be reset 
                    self.selected_piece = None;

                // There is no piece selected, so it selects the piece the player is currently hovering over
                } else if let Some(piece) = self.pieces.iter().find(|piece| piece.position == hovered_piece_pos && piece.piece_type != PieceType::Dead) {
                    self.selected_piece = match self.selected_piece.is_none() {
                        true => Some(piece.position),
                        false => None,
                    };

                }
            }
        }

    }

    fn set_new_stage(&mut self) -> Option<Stages> {
        None
    }
}

//JS function to send the board
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn send_board(board_string: JsObject);
}


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

#[cfg(target_arch = "wasm32")]
pub fn chess_board_from_bin(bin: [u8; 256]) -> [Piece; 32] {
    let bin_chunks = bin.chunks(8);
    bin_chunks.map(|chunk| Piece::from_bin(chunk)).collect::<Vec<Piece>>().as_slice().try_into().unwrap()

}
