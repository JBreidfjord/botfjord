from __future__ import annotations

from dataclasses import dataclass

from chess import Board


@dataclass
class Branch:
    prior: float
    visit_count: int = 0
    total_value: float = 0


class Node:
    def __init__(self, state: Board, priors, value, parent=None, last_move=None):
        self.state = state
        self.value = value
        self.parent = parent
        self.last_move = last_move
        self.total_visit_count = 1
        self.branches = {}
        self.children = {}

        for move in list(state.legal_moves):
            self.branches[move] = Branch(priors[move])


class MCTS:
    def __init__(
        self,
        evaluation_function: function,
        prior_function: function,
        num_rounds: int = 10000,
        temperature: float = 5.0,
    ):
        self._eval_func = evaluation_function
        self._prior_func = prior_function
        self.num_rounds = num_rounds
        self._c = temperature
