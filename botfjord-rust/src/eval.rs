use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square};
use ordered_float::OrderedFloat;
use std::collections::HashMap;

pub struct Evaluator {
    piece_value_map: HashMap<Piece, f32>,
    outer_ring: Vec<Square>,
    mid_ring: Vec<Square>,
    inner_ring: Vec<Square>,
    center: Vec<Square>,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let mut pvm = HashMap::new();
        pvm.insert(Piece::Pawn, 1.0);
        pvm.insert(Piece::Bishop, 3.0);
        pvm.insert(Piece::Knight, 3.0);
        pvm.insert(Piece::Rook, 5.0);
        pvm.insert(Piece::Queen, 9.0);
        Evaluator {
            piece_value_map: pvm,
            outer_ring: BitBoard::new(18411139144890810879).collect(),
            mid_ring: BitBoard::new(35538699412471296).collect(),
            inner_ring: BitBoard::new(66125924401152).collect(),
            center: BitBoard::new(103481868288).collect(),
        }
    }

    pub fn evaluate(&self, state: Board) -> f32 {
        if state.status() == BoardStatus::Checkmate {
            return -39.0;
        }

        let mut value = 0.0;
        let black = state.color_combined(Color::Black);
        let white = state.color_combined(Color::White);
        let pawns = state.pieces(Piece::Pawn);
        let bishops = state.pieces(Piece::Bishop);
        let knights = state.pieces(Piece::Knight);
        let rooks = state.pieces(Piece::Rook);
        let queens = state.pieces(Piece::Queen);

        value -= (black & pawns).popcnt() as f32 * self.piece_value_map[&Piece::Pawn];
        value -= (black & bishops).popcnt() as f32 * self.piece_value_map[&Piece::Bishop];
        value -= (black & knights).popcnt() as f32 * self.piece_value_map[&Piece::Knight];
        value -= (black & rooks).popcnt() as f32 * self.piece_value_map[&Piece::Rook];
        value -= (black & queens).popcnt() as f32 * self.piece_value_map[&Piece::Queen];
        value += (white & pawns).popcnt() as f32 * self.piece_value_map[&Piece::Pawn];
        value += (white & bishops).popcnt() as f32 * self.piece_value_map[&Piece::Bishop];
        value += (white & knights).popcnt() as f32 * self.piece_value_map[&Piece::Knight];
        value += (white & rooks).popcnt() as f32 * self.piece_value_map[&Piece::Rook];
        value += (white & queens).popcnt() as f32 * self.piece_value_map[&Piece::Queen];

        // Value for center control
        for action in MoveGen::new_legal(&state) {
            if self.center.contains(&action.get_dest()) {
                value += 0.25
            }
        }
        // Flip board with null move to get opponent's center control
        // Skipped if currently in check
        if state.checkers().popcnt() == 0 {
            let opp_state = state.null_move().unwrap();
            for action in MoveGen::new_legal(&opp_state) {
                if self.center.contains(&action.get_dest()) {
                    value -= 0.25
                }
            }
        }

        // Value for pushing king to outside in endgame
        if black.popcnt() <= 4 {
            let king = state.king_square(Color::Black);
            if self.center.contains(&king) {
                value -= 0.5
            } else if self.inner_ring.contains(&king) {
                value -= 0.25
            } else if self.mid_ring.contains(&king) {
                value += 0.25
            } else if self.outer_ring.contains(&king) {
                value += 0.5
            }
        }
        if white.popcnt() <= 4 {
            let king = state.king_square(Color::White);
            if self.center.contains(&king) {
                value += 0.5
            } else if self.inner_ring.contains(&king) {
                value += 0.25
            } else if self.mid_ring.contains(&king) {
                value -= 0.25
            } else if self.outer_ring.contains(&king) {
                value -= 0.5
            }
        }

        if state.side_to_move() == Color::Black {
            value = -value
        }

        // Value loss for each checker
        if state.checkers().popcnt() > 0 {
            value -= 0.75 * state.checkers().popcnt() as f32
        }

        value
    }

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
