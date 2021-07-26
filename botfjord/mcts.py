from __future__ import annotations

from dataclasses import dataclass

from chess import Board, Move


@dataclass
class Branch:
    prior: float
    visit_count: int = 0
    total_value: float = 0


class Node:
    def __init__(
        self,
        state: Board,
        priors: dict[Move, float],
        value: float,
        parent: Node = None,
        last_move: Move = None,
    ):
        """
        A Monte Carlo Tree Search node\n
        Args:
            state: Board object for current game state
            priors: dict of float values for all legal moves from current state
            value: The calculated value of the current state
            parent: Optional; The Node leading to this Node
            last_move: Optional; The move leading from parent state to current state
        """
        self.state = state
        self.value = value
        self.parent = parent
        self.last_move = last_move
        self.total_visit_count = 1
        self.branches: dict[Move, Branch] = {}
        self.children: dict[Move, Node] = {}

        for move in list(state.legal_moves):
            self.branches[move] = Branch(priors[move])

    def moves(self):
        """Returns a list of all possible moves from this node"""
        return list(self.branches.keys())

    def add_child(self, move: Move, child_node: Node):
        """Adds a new child node\n
        Raises exception if child exists"""
        if self.children.get(move) is not None:
            raise ValueError("Child already exists")
        self.children[move] = child_node

    def has_child(self, move: Move):
        """Returns bool for existence of given child node"""
        return move in self.children

    def get_child(self, move: Move):
        """Returns child node for given move\n
        Raise exception if child does not exist"""
        return self.children[move]

    def expected_value(self, move: Move):
        """Returns expected value for given move\n
        Expected value is the average value of past visits to that branch"""
        branch = self.branches[move]
        if branch.visit_count == 0:
            return 0.0
        return branch.total_value / branch.visit_count

    def prior(self, move: Move):
        """Returns prior value for given move"""
        return self.branches[move].prior

    def visit_count(self, move: Move):
        """Returns visit count of branch for given move"""
        branch = self.branches.get(move)
        return branch.visit_count if branch is not None else 0

    def record_visit(self, move: Move, value: float):
        """Updates the branch for given move"""
        self.branches[move].visit_count += 1
        self.branches[move].total_value += value
        self.total_visit_count += 1

    def check_visit_counts(self, num_rounds: int):
        """Returns True if the most visited branch is guaranteed to be selected
        for the given number of search rounds"""
        visit_counts = sorted(self.branches.values(), key=lambda b: b.visit_count)
        remaining_rounds = num_rounds - self.total_visit_count
        return visit_counts[-1].visit_count >= visit_counts[-2].visit_count + remaining_rounds


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
