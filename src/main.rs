mod eval;
mod genetic;
mod mcts;

use crate::genetic::run_ga;

fn main() {
    run_ga(10, 0.4, 0.5, 100, 250);
}
