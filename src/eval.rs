use chess::{Board, BoardStatus, Color, Piece, Square, ALL_COLORS, ALL_PIECES, ALL_SQUARES};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Evaluator {
    pub early_maps: HashMap<Color, HashMap<Piece, HashMap<Square, isize>>>,
    pub end_maps: HashMap<Color, HashMap<Piece, HashMap<Square, isize>>>,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let pvm = create_map();

        let mut early_maps = HashMap::new();
        let mut end_maps = HashMap::new();

        for color in ALL_COLORS {
            let mut early_color_map = HashMap::new();
            let mut end_color_map = HashMap::new();

            for piece in ALL_PIECES {
                let mut early_piece_map = HashMap::new();
                let mut end_piece_map = HashMap::new();

                for square in ALL_SQUARES {
                    early_piece_map.insert(square, pvm[&piece]);
                    end_piece_map.insert(square, pvm[&piece]);
                }
                early_color_map.insert(piece, early_piece_map);
                end_color_map.insert(piece, end_piece_map);
            }
            early_maps.insert(color, early_color_map);
            end_maps.insert(color, end_color_map);
        }

        Evaluator {
            early_maps,
            end_maps,
        }
    }

    pub fn empty() -> Evaluator {
        Evaluator {
            early_maps: HashMap::new(),
            end_maps: HashMap::new(),
        }
    }

    pub fn evaluate(&self, state: Board) -> f32 {
        if state.status() == BoardStatus::Checkmate {
            return -3200.0;
        }

        // Use material count to determine game phase
        let taper = match state.combined().popcnt() {
            1..=6 => 1.0,
            7..=12 => 0.75,
            13..=22 => 0.5,
            23..=28 => 0.25,
            29..=32 => 0.0,
            _ => 0.5,
        };

        let mut early_value = 0;
        let mut end_value = 0;

        for color in ALL_COLORS {
            let early_color_map = &self.early_maps[&color];
            let end_color_map = &self.end_maps[&color];
            let color_bb = state.color_combined(color);

            for piece in ALL_PIECES {
                let early_piece_map = &early_color_map[&piece];
                let end_piece_map = &end_color_map[&piece];
                let piece_bb = state.pieces(piece);

                for square in color_bb & piece_bb {
                    if color == state.side_to_move() {
                        early_value += early_piece_map[&square];
                        end_value += end_piece_map[&square];
                    } else {
                        early_value -= early_piece_map[&square];
                        end_value -= end_piece_map[&square];
                    }
                }
            }
        }

        let mut value = (taper * end_value as f32) + ((1.0 - taper) * early_value as f32);

        // Remove value for number of checkers
        value -= match state.checkers().popcnt() {
            0 => 0.0,
            1 => 0.25,
            2 => 0.75,
            3 => 1.5,
            _ => 39.0,
        };

        value
    }
}

fn create_map() -> HashMap<Piece, isize> {
    let mut pvm = HashMap::new();
    pvm.insert(Piece::Pawn, 1);
    pvm.insert(Piece::Bishop, 3);
    pvm.insert(Piece::Knight, 3);
    pvm.insert(Piece::Rook, 5);
    pvm.insert(Piece::Queen, 9);
    pvm.insert(Piece::King, 0);

    pvm
}
