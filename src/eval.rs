use chess::{Board, BoardStatus, Color, Piece, Square, ALL_COLORS, ALL_PIECES, ALL_SQUARES};
use std::{
    collections::HashMap,
    fs::File,
    io::{prelude::*, Read},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct Evaluator {
    pub early_maps: HashMap<Color, HashMap<Piece, HashMap<Square, isize>>>,
    pub end_maps: HashMap<Color, HashMap<Piece, HashMap<Square, isize>>>,
    value_map: HashMap<Piece, isize>,
}

impl Evaluator {
    #[allow(dead_code)]
    pub fn new() -> Evaluator {
        let mut early_maps = HashMap::new();
        let mut end_maps = HashMap::new();

        for color in ALL_COLORS {
            let mut early_color_map = HashMap::new();
            let mut end_color_map = HashMap::new();

            for piece in ALL_PIECES {
                let mut early_piece_map = HashMap::new();
                let mut end_piece_map = HashMap::new();

                for square in ALL_SQUARES {
                    early_piece_map.insert(square, 0);
                    end_piece_map.insert(square, 0);
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
            value_map: create_map(),
        }
    }

    pub fn empty() -> Evaluator {
        Evaluator {
            early_maps: HashMap::new(),
            end_maps: HashMap::new(),
            value_map: create_map(),
        }
    }

    #[allow(dead_code)]
    pub fn evaluate(&self, state: Board) -> f32 {
        if state.status() == BoardStatus::Checkmate {
            return -30000.0;
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

        // Start at 10 centipawns for side to move bonus
        let mut early_value = 10;
        let mut end_value = 10;

        for color in ALL_COLORS {
            let early_color_map = &self.early_maps[&color];
            let end_color_map = &self.end_maps[&color];
            let color_bb = state.color_combined(color);

            for piece in ALL_PIECES {
                let early_piece_map = &early_color_map[&piece];
                let end_piece_map = &end_color_map[&piece];
                let piece_bb = state.pieces(piece);
                let piece_val = self.value_map[&piece];

                for square in color_bb & piece_bb {
                    if color == state.side_to_move() {
                        early_value += piece_val + early_piece_map[&square];
                        end_value += piece_val + end_piece_map[&square];
                    } else {
                        early_value -= piece_val + early_piece_map[&square];
                        end_value -= piece_val + end_piece_map[&square];
                    }
                }
            }
        }

        let value = (taper * end_value as f32) + ((1.0 - taper) * early_value as f32);

        value
    }

    pub fn write(&self, path: &str) {
        let path = Path::new(path);
        let mut file = File::create(path).unwrap();
        for color in ALL_COLORS {
            let early_color_map = &self.early_maps[&color];
            let end_color_map = &self.end_maps[&color];
            for piece in ALL_PIECES {
                let early_piece_map = &early_color_map[&piece];
                let end_piece_map = &end_color_map[&piece];

                for square in ALL_SQUARES {
                    file.write_all(&early_piece_map[&square].to_ne_bytes())
                        .unwrap();
                    file.write_all(&end_piece_map[&square].to_ne_bytes())
                        .unwrap();
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn read(path: &str) -> Evaluator {
        // collect all from file then iterate through colors, piece, and squares to input
        let mut evaluator = Evaluator::empty();
        let mut file = File::open(path).unwrap();
        let mut data = vec![];
        file.read_to_end(&mut data).unwrap();
        let mut i = 0;
        for color in ALL_COLORS {
            let mut early_color_map = HashMap::new();
            let mut end_color_map = HashMap::new();
            for piece in ALL_PIECES {
                let mut early_piece_map = HashMap::new();
                let mut end_piece_map = HashMap::new();

                for square in ALL_SQUARES {
                    early_piece_map.insert(square, data[i] as isize);
                    i += 8;
                    end_piece_map.insert(square, data[i] as isize);
                    i += 8;
                }
                early_color_map.insert(piece, early_piece_map);
                end_color_map.insert(piece, end_piece_map);
            }
            evaluator.early_maps.insert(color, early_color_map);
            evaluator.end_maps.insert(color, end_color_map);
        }

        evaluator
    }

    #[allow(dead_code)]
    pub fn create(white_vec: Vec<isize>, black_vec: Vec<isize>) -> Evaluator {
        let mut evaluator = Evaluator::empty();

        for color in ALL_COLORS {
            let mut i = 0;
            let vec = if color == Color::White {
                white_vec.clone()
            } else {
                black_vec.clone()
            };

            let mut early_color_map = HashMap::new();
            let mut end_color_map = HashMap::new();
            for piece in ALL_PIECES {
                let mut early_piece_map = HashMap::new();
                let mut end_piece_map = HashMap::new();

                for square in ALL_SQUARES {
                    early_piece_map.insert(square, vec[i]);
                    i += 1;

                    end_piece_map.insert(square, vec[i]);
                    i += 1;
                }
                early_color_map.insert(piece, early_piece_map);
                end_color_map.insert(piece, end_piece_map);
            }
            evaluator.early_maps.insert(color, early_color_map);
            evaluator.end_maps.insert(color, end_color_map);
        }

        evaluator
    }
}

fn create_map() -> HashMap<Piece, isize> {
    let mut pvm = HashMap::new();
    pvm.insert(Piece::Pawn, 100);
    pvm.insert(Piece::Bishop, 333);
    pvm.insert(Piece::Knight, 305);
    pvm.insert(Piece::Rook, 563);
    pvm.insert(Piece::Queen, 950);
    pvm.insert(Piece::King, 20000);

    pvm
}
