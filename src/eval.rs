use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square};
use ordered_float::OrderedFloat;
use std::collections::HashMap;

const PAWN_PHASE: usize = 0;
const KNIGHT_PHASE: usize = 1;
const BISHOP_PHASE: usize = 1;
const ROOK_PHASE: usize = 2;
const QUEEN_PHASE: usize = 4;
const TOTAL_PHASE: usize =
    PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

pub struct Evaluator {
    piece_value_map: HashMap<Piece, f32>,
    eg_piece_value_map: HashMap<Piece, f32>,
    piece_square_table: HashMap<Piece, [f32; 64]>,
    eg_piece_square_table: HashMap<Piece, [f32; 64]>,
    // piece_square_table: HashMap<(Color, Piece, Square), f32>,
    //     outer_ring: Vec<Square>,
    //     mid_ring: Vec<Square>,
    //     inner_ring: Vec<Square>,
    //     center: Vec<Square>
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let mut pvm = HashMap::new();
        pvm.insert(Piece::Pawn, 0.82);
        pvm.insert(Piece::Knight, 3.37);
        pvm.insert(Piece::Bishop, 3.65);
        pvm.insert(Piece::Rook, 4.77);
        pvm.insert(Piece::Queen, 10.25);
        pvm.insert(Piece::King, 0.0);

        let mut eg_pvm = HashMap::new();
        eg_pvm.insert(Piece::Pawn, 0.94);
        eg_pvm.insert(Piece::Knight, 2.81);
        eg_pvm.insert(Piece::Bishop, 2.97);
        eg_pvm.insert(Piece::Rook, 5.12);
        eg_pvm.insert(Piece::Queen, 9.36);
        eg_pvm.insert(Piece::King, 0.0);

        let (pst, eg_pst) = create_pst();

        Evaluator {
            piece_value_map: pvm,
            eg_piece_value_map: eg_pvm,
            piece_square_table: pst,
            eg_piece_square_table: eg_pst,
            // outer_ring: BitBoard::new(18411139144890810879).collect(),
            // mid_ring: BitBoard::new(35538699412471296).collect(),
            // inner_ring: BitBoard::new(66125924401152).collect(),
            // center: BitBoard::new(103481868288).collect(),
        }
    }

    pub fn evaluate(&self, state: Board) -> f32 {
        if state.status() == BoardStatus::Checkmate {
            return -39.0;
        }

        // Use material count to determine game phase
        let mut phase = TOTAL_PHASE;
        // phase -= state.pieces(Piece::Pawn).popcnt() as usize * PAWN_PHASE;
        phase -= state.pieces(Piece::Knight).popcnt() as usize * KNIGHT_PHASE;
        phase -= state.pieces(Piece::Bishop).popcnt() as usize * BISHOP_PHASE;
        phase -= state.pieces(Piece::Rook).popcnt() as usize * ROOK_PHASE;
        phase -= state.pieces(Piece::Queen).popcnt() as usize * QUEEN_PHASE;
        phase = (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
        let taper = (phase / 256) as f32;

        // Value bonus for side to move
        let mut value = 0.1;

        for color in chess::ALL_COLORS {
            let color_bb = state.color_combined(color);
            let color_mult = match color == state.side_to_move() {
                true => 1.0,
                false => -1.0,
            };

            for piece in chess::ALL_PIECES {
                let piece_bb = color_bb & state.pieces(piece);
                let piece_value = self.piece_value_map.get(&piece).unwrap();
                let eg_piece_value = self.eg_piece_value_map.get(&piece).unwrap();
                let piece_square_table = self.piece_square_table.get(&piece).unwrap();
                let eg_piece_square_table = self.eg_piece_square_table.get(&piece).unwrap();

                let tapered_piece_value = (piece_value * (1.0 - taper)) + (eg_piece_value * taper);

                for square in piece_bb {
                    let i = if color == Color::White {
                        square.to_index()
                    } else {
                        // XOR to flip board
                        square.to_index() ^ 56
                    };
                    let square_value = (piece_square_table[i] * (1.0 - taper))
                        + (eg_piece_square_table[i] * taper);

                    value += color_mult * (tapered_piece_value + square_value);
                }
            }
        }

        value
    }

    // pub fn evaluate(&self, state: Board) -> f32 {
    //     if state.status() == BoardStatus::Checkmate {
    //         return -39.0;
    //     }

    //     let mut value = 0.0;
    //     let black = state.color_combined(Color::Black);
    //     let white = state.color_combined(Color::White);
    //     let pawns = state.pieces(Piece::Pawn);
    //     let bishops = state.pieces(Piece::Bishop);
    //     let knights = state.pieces(Piece::Knight);
    //     let rooks = state.pieces(Piece::Rook);
    //     let queens = state.pieces(Piece::Queen);

    //     value -= (black & pawns).popcnt() as f32 * self.piece_value_map[&Piece::Pawn];
    //     value -= (black & bishops).popcnt() as f32 * self.piece_value_map[&Piece::Bishop];
    //     value -= (black & knights).popcnt() as f32 * self.piece_value_map[&Piece::Knight];
    //     value -= (black & rooks).popcnt() as f32 * self.piece_value_map[&Piece::Rook];
    //     value -= (black & queens).popcnt() as f32 * self.piece_value_map[&Piece::Queen];
    //     value += (white & pawns).popcnt() as f32 * self.piece_value_map[&Piece::Pawn];
    //     value += (white & bishops).popcnt() as f32 * self.piece_value_map[&Piece::Bishop];
    //     value += (white & knights).popcnt() as f32 * self.piece_value_map[&Piece::Knight];
    //     value += (white & rooks).popcnt() as f32 * self.piece_value_map[&Piece::Rook];
    //     value += (white & queens).popcnt() as f32 * self.piece_value_map[&Piece::Queen];

    //     // Value for pushing king to outside in endgame
    //     if black.popcnt() <= 4 {
    //         let king = state.king_square(Color::Black);
    //         if self.center.contains(&king) {
    //             value -= 0.5
    //         } else if self.inner_ring.contains(&king) {
    //             value -= 0.25
    //         } else if self.mid_ring.contains(&king) {
    //             value += 0.25
    //         } else if self.outer_ring.contains(&king) {
    //             value += 0.5
    //         }
    //     }
    //     if white.popcnt() <= 4 {
    //         let king = state.king_square(Color::White);
    //         if self.center.contains(&king) {
    //             value += 0.5
    //         } else if self.inner_ring.contains(&king) {
    //             value += 0.25
    //         } else if self.mid_ring.contains(&king) {
    //             value -= 0.25
    //         } else if self.outer_ring.contains(&king) {
    //             value -= 0.5
    //         }
    //     }

    //     if state.side_to_move() == Color::Black {
    //         value = -value
    //     }

    //     // Remove value for pinned pieces
    //     let pinned: Vec<_> = state.pinned().collect();
    //     for square in pinned {
    //         let piece = state.piece_on(square).unwrap();
    //         if piece != Piece::King {
    //             value -= self.piece_value_map[&piece]
    //         }
    //     }

    //     // Value for center control
    //     for action in MoveGen::new_legal(&state) {
    //         if self.center.contains(&action.get_dest()) {
    //             value += 0.25
    //         }
    //     }
    //     // Flip board with null move to get opponent's info
    //     // Skipped if currently in check
    //     if state.checkers().popcnt() == 0 {
    //         let opp_state = state.null_move().unwrap();
    //         assert_ne!(state, opp_state);

    //         for action in MoveGen::new_legal(&opp_state) {
    //             if self.center.contains(&action.get_dest()) {
    //                 value -= 0.25
    //             }
    //         }
    //         let pinned: Vec<_> = state.pinned().collect();
    //         for square in pinned {
    //             let piece = state.piece_on(square).unwrap();
    //             if piece != Piece::King {
    //                 value += self.piece_value_map[&piece]
    //             }
    //         }
    //     } else {
    //         // Value loss for each checker
    //         value -= 0.75 * state.checkers().popcnt() as f32
    //     }

    //     value
    // }

    pub fn priors(&self, state: Board) -> HashMap<ChessMove, f32> {
        let mut priors = HashMap::new();

        let score = |state: Board| {
            if state.status() == BoardStatus::Checkmate {
                return -16.0;
            }
            let piece_diff = state.color_combined(Color::White).popcnt() as f32
                - state.color_combined(Color::Black).popcnt() as f32;
            match state.side_to_move() {
                Color::White => piece_diff,
                Color::Black => -piece_diff,
            }
        };

        for action in MoveGen::new_legal(&state) {
            let new_state = state.make_move_new(action);
            priors.insert(action, score(new_state) + 0.0000001);
        }

        if priors.is_empty() {
            return priors;
        }

        let abs_min = priors
            .values()
            .min_by_key(|v| OrderedFloat(**v))
            .unwrap()
            .abs();
        let max = (priors.values().max_by_key(|v| OrderedFloat(**v)).unwrap() + abs_min) * 1.25;
        let mut new_priors = HashMap::new();
        for (action, value) in priors.iter() {
            new_priors.insert(*action, max - (value + abs_min));
        }

        let sum: f32 = new_priors.values().sum();
        let norm_factor = 1.0 / (sum + 0.0000001);
        for (action, value) in new_priors.iter() {
            priors.insert(*action, value * norm_factor);
        }

        priors
    }
}

fn create_pst() -> (HashMap<Piece, [f32; 64]>, HashMap<Piece, [f32; 64]>) {
    let pawn_table = [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.35, -0.01, -0.2, -0.23, -0.15, 0.24, 0.38,
        -0.22, -0.26, -0.04, -0.04, -0.1, 0.03, 0.03, 0.33, -0.12, -0.27, -0.02, -0.05, 0.12, 0.17,
        0.06, 0.1, -0.25, -0.14, 0.13, 0.06, 0.21, 0.23, 0.12, 0.17, -0.23, -0.06, 0.07, 0.26,
        0.31, 0.65, 0.56, 0.25, -0.2, 0.98, 1.34, 0.61, 0.95, 0.68, 1.26, 0.34, -0.11, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];

    let eg_pawn_table = [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.13, 0.08, 0.08, 0.1, 0.13, 0.0, 0.02, -0.07,
        0.04, 0.07, -0.06, 0.01, 0.0, -0.05, -0.01, -0.08, 0.13, 0.09, -0.03, -0.07, -0.07, -0.08,
        0.03, -0.01, 0.32, 0.24, 0.13, 0.05, -0.02, 0.04, 0.17, 0.17, 0.94, 1.0, 0.85, 0.67, 0.56,
        0.53, 0.82, 0.84, 1.78, 1.73, 1.58, 1.34, 1.47, 1.32, 1.65, 1.87, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0,
    ];

    let knight_table = [
        -1.05, -0.21, -0.58, -0.33, -0.17, -0.28, -0.19, -0.23, -0.29, -0.53, -0.12, -0.03, -0.01,
        0.18, -0.14, -0.19, -0.23, -0.09, 0.12, 0.1, 0.19, 0.17, 0.25, -0.16, -0.13, 0.04, 0.16,
        0.13, 0.28, 0.19, 0.21, -0.08, -0.09, 0.17, 0.19, 0.53, 0.37, 0.69, 0.18, 0.22, -0.47, 0.6,
        0.37, 0.65, 0.84, 1.29, 0.73, 0.44, -0.73, -0.41, 0.72, 0.36, 0.23, 0.62, 0.07, -0.17,
        -1.67, -0.89, -0.34, -0.49, 0.61, -0.97, -0.15, -1.07,
    ];

    let eg_knight_table = [
        -0.29, -0.51, -0.23, -0.15, -0.22, -0.18, -0.5, -0.64, -0.42, -0.2, -0.1, -0.05, -0.02,
        -0.2, -0.23, -0.44, -0.23, -0.03, -0.01, 0.15, 0.1, -0.03, -0.2, -0.22, -0.18, -0.06, 0.16,
        0.25, 0.16, 0.17, 0.04, -0.18, -0.17, 0.03, 0.22, 0.22, 0.22, 0.11, 0.08, -0.18, -0.24,
        -0.2, 0.1, 0.09, -0.01, -0.09, -0.19, -0.41, -0.25, -0.08, -0.25, -0.02, -0.09, -0.25,
        -0.24, -0.52, -0.58, -0.38, -0.13, -0.28, -0.31, -0.27, -0.63, -0.99,
    ];

    let bishop_table = [
        -0.33, -0.03, -0.14, -0.21, -0.13, -0.12, -0.39, -0.21, 0.04, 0.15, 0.16, 0.0, 0.07, 0.21,
        0.33, 0.01, 0.0, 0.15, 0.15, 0.15, 0.14, 0.27, 0.18, 0.1, -0.06, 0.13, 0.13, 0.26, 0.34,
        0.12, 0.1, 0.04, -0.04, 0.05, 0.19, 0.5, 0.37, 0.37, 0.07, -0.02, -0.16, 0.37, 0.43, 0.4,
        0.35, 0.5, 0.37, -0.02, -0.26, 0.16, -0.18, -0.13, 0.3, 0.59, 0.18, -0.47, -0.29, 0.04,
        -0.82, -0.37, -0.25, -0.42, 0.07, -0.08,
    ];

    let eg_bishop_table = [
        -0.23, -0.09, -0.23, -0.05, -0.09, -0.16, -0.05, -0.17, -0.14, -0.18, -0.07, -0.01, 0.04,
        -0.09, -0.15, -0.27, -0.12, -0.03, 0.08, 0.1, 0.13, 0.03, -0.07, -0.15, -0.06, 0.03, 0.13,
        0.19, 0.07, 0.1, -0.03, -0.09, -0.03, 0.09, 0.12, 0.09, 0.14, 0.1, 0.03, 0.02, 0.02, -0.08,
        0.0, -0.01, -0.02, 0.06, 0.0, 0.04, -0.08, -0.04, 0.07, -0.12, -0.03, -0.13, -0.04, -0.14,
        -0.14, -0.21, -0.11, -0.08, -0.07, -0.09, -0.17, -0.24,
    ];

    let rook_table = [
        -0.19, -0.13, 0.01, 0.17, 0.16, 0.07, -0.37, -0.26, -0.44, -0.16, -0.2, -0.09, -0.01, 0.11,
        -0.06, -0.71, -0.45, -0.25, -0.16, -0.17, 0.03, 0.0, -0.05, -0.33, -0.36, -0.26, -0.12,
        -0.01, 0.09, -0.07, 0.06, -0.23, -0.24, -0.11, 0.07, 0.26, 0.24, 0.35, -0.08, -0.2, -0.05,
        0.19, 0.26, 0.36, 0.17, 0.45, 0.61, 0.16, 0.27, 0.32, 0.58, 0.62, 0.8, 0.67, 0.26, 0.44,
        0.32, 0.42, 0.32, 0.51, 0.63, 0.09, 0.31, 0.43,
    ];

    let eg_rook_table = [
        -0.09, 0.02, 0.03, -0.01, -0.05, -0.13, 0.04, -0.2, -0.06, -0.06, 0.0, 0.02, -0.09, -0.09,
        -0.11, -0.03, -0.04, 0.0, -0.05, -0.01, -0.07, -0.12, -0.08, -0.16, 0.03, 0.05, 0.08, 0.04,
        -0.05, -0.06, -0.08, -0.11, 0.04, 0.03, 0.13, 0.01, 0.02, 0.01, -0.01, 0.02, 0.07, 0.07,
        0.07, 0.05, 0.04, -0.03, -0.05, -0.03, 0.11, 0.13, 0.13, 0.11, -0.03, 0.03, 0.08, 0.03,
        0.13, 0.1, 0.18, 0.15, 0.12, 0.12, 0.08, 0.05,
    ];

    let queen_table = [
        -0.01, -0.18, -0.09, 0.1, -0.15, -0.25, -0.31, -0.5, -0.35, -0.08, 0.11, 0.02, 0.08, 0.15,
        -0.03, 0.01, -0.14, 0.02, -0.11, -0.02, -0.05, 0.02, 0.14, 0.05, -0.09, -0.26, -0.09, -0.1,
        -0.02, -0.04, 0.03, -0.03, -0.27, -0.27, -0.16, -0.16, -0.01, 0.17, -0.02, 0.01, -0.13,
        -0.17, 0.07, 0.08, 0.29, 0.56, 0.47, 0.57, -0.24, -0.39, -0.05, 0.01, -0.16, 0.57, 0.28,
        0.54, -0.28, 0.0, 0.29, 0.12, 0.59, 0.44, 0.43, 0.45,
    ];

    let eg_queen_table = [
        -0.33, -0.28, -0.22, -0.43, -0.05, -0.32, -0.2, -0.41, -0.22, -0.23, -0.3, -0.16, -0.16,
        -0.23, -0.36, -0.32, -0.16, -0.27, 0.15, 0.06, 0.09, 0.17, 0.1, 0.05, -0.18, 0.28, 0.19,
        0.47, 0.31, 0.34, 0.39, 0.23, 0.03, 0.22, 0.24, 0.45, 0.57, 0.4, 0.57, 0.36, -0.2, 0.06,
        0.09, 0.49, 0.47, 0.35, 0.19, 0.09, -0.17, 0.2, 0.32, 0.41, 0.58, 0.25, 0.3, 0.0, -0.09,
        0.22, 0.22, 0.27, 0.27, 0.19, 0.1, 0.2,
    ];

    let king_table = [
        -0.15, 0.36, 0.12, -0.54, 0.08, -0.28, 0.24, 0.14, 0.01, 0.07, -0.08, -0.64, -0.43, -0.16,
        0.09, 0.08, -0.14, -0.14, -0.22, -0.46, -0.44, -0.3, -0.15, -0.27, -0.49, -0.01, -0.27,
        -0.39, -0.46, -0.44, -0.33, -0.51, -0.17, -0.2, -0.12, -0.27, -0.3, -0.25, -0.14, -0.36,
        -0.09, 0.24, 0.02, -0.16, -0.2, 0.06, 0.22, -0.22, 0.29, -0.01, -0.2, -0.07, -0.08, -0.04,
        -0.38, -0.29, -0.65, 0.23, 0.16, -0.15, -0.56, -0.34, 0.02, 0.13,
    ];

    let eg_king_table = [
        -0.53, -0.34, -0.21, -0.11, -0.28, -0.14, -0.24, -0.43, -0.27, -0.11, 0.04, 0.13, 0.14,
        0.04, -0.05, -0.17, -0.19, -0.03, 0.11, 0.21, 0.23, 0.16, 0.07, -0.09, -0.18, -0.04, 0.21,
        0.24, 0.27, 0.23, 0.09, -0.11, -0.08, 0.22, 0.24, 0.27, 0.26, 0.33, 0.26, 0.03, 0.1, 0.17,
        0.23, 0.15, 0.2, 0.45, 0.44, 0.13, -0.12, 0.17, 0.14, 0.17, 0.17, 0.38, 0.23, 0.11, -0.74,
        -0.35, -0.18, -0.18, -0.11, 0.15, 0.04, -0.17,
    ];

    let mut pst = HashMap::new();
    let mut eg_pst = HashMap::new();

    pst.insert(Piece::Pawn, pawn_table);
    pst.insert(Piece::Knight, knight_table);
    pst.insert(Piece::Bishop, bishop_table);
    pst.insert(Piece::Rook, rook_table);
    pst.insert(Piece::Queen, queen_table);
    pst.insert(Piece::King, king_table);

    eg_pst.insert(Piece::Pawn, eg_pawn_table);
    eg_pst.insert(Piece::Knight, eg_knight_table);
    eg_pst.insert(Piece::Bishop, eg_bishop_table);
    eg_pst.insert(Piece::Rook, eg_rook_table);
    eg_pst.insert(Piece::Queen, eg_queen_table);
    eg_pst.insert(Piece::King, eg_king_table);

    (pst, eg_pst)
}
