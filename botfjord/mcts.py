from __future__ import annotations

import timeit
from dataclasses import dataclass
from math import log, sqrt

from chess import Board, Move
from tqdm.auto import tqdm


@dataclass
class Branch:
    prior: float
    visit_count: int = 0
    total_value: float = 0.0


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
        temperature: float = sqrt(2),
        verbose: bool = False,
    ):
        """
        Monte Carlo Tree Search\n
        Args:
            evaluation_function & prior_function:
                The functions to calculate the value of game states.\n\t
                Prior function should be a small, fast calculation 
                used to quickly determine branch priority without searching.
                Function should return dict[Move, float].\n\t
                Evaluation function is the main function used for
                calculating the value of a state.
                Function should return float.
            num_rounds: Maximum number of search rounds before returning the selected move.
            temperature: Constant value that determines the exploration/exploitation 
            trade-off when searching.
        """
        self._eval_func = evaluation_function
        self._prior_func = prior_function
        self.num_rounds = num_rounds
        self._c = temperature
        self.verbose = verbose

    def set_functions(self, evaluation_function: function = None, prior_function: function = None):
        if isinstance(evaluation_function, function):
            self._eval_func = evaluation_function
        if isinstance(prior_function, function):
            self._prior_func = prior_function

    def set_temperature(self, temperature: float):
        self._c = temperature

    def create_node(self, game_state: Board, move: Move = None, parent: Node = None):
        """
        Creates a new MCTS Node and calculates value and priors\n
        Node is added as a child if parent node is given
        """
        eval_start_time = timeit.default_timer() if self.verbose else 0.0

        priors = self._prior_func(game_state)
        value = self._eval_func(game_state)

        if self.verbose:
            self.eval_time += timeit.default_timer() - eval_start_time

        node = Node(game_state, value, priors, parent, move)
        if parent is not None and not game_state.is_game_over():
            parent.add_child(move, node)

        return node

    def select_branch(self, node: Node):
        """
        Selects next branch to search based on the PUCT formula:
            q + c * p * (sqrt( ln(N) / n ))
            \n\tWhere:
            q is the expected value of the node;\n
            c is the search temperature, a constant;\n
            p is the prior value assigned to the node;\n
            N is the total visit count of the parent node;\n
            n is the total visit count of the node;
        """
        total_n = node.total_visit_count

        def score_branch(move: Move):
            q = node.expected_value(move)
            p = node.prior(move)
            n = node.visit_count(move)
            return q + self._c * p * (sqrt(log(total_n) / (n + 1)))

        return max(node.moves(), key=score_branch)

    def select_move(self, game_state: Board):
        """The core of the MCTS class.
        Starts a search from the given Board and returns the selected Move."""
        # Return early if only 1 legal move available
        if game_state.legal_moves.count() == 1:
            return next(game_state.generate_legal_moves())

        if self.verbose:
            self.eval_time = 0.0
            start_time = timeit.default_timer()
        t = tqdm(range(self.num_rounds), leave=False, ncols=80, disable=not self.verbose)

        root = self.create_node(game_state)
        for _ in t:
            node = root
            next_move = self.select_branch(node)

            while node.has_child(next_move):
                node: Node = node.get_child(next_move)
                next_move = self.select_branch(node)

            new_state = node.state.copy(stack=False)
            new_state.push(next_move)
            child_node = self.create_node(new_state, move=next_move, parent=node)

            move = next_move
            value = -child_node.value
            while node is not None:
                node.record_visit(move, value)
                move = node.last_move
                node = node.parent
                value = -value

            if self.verbose:
                top_5 = sorted(root.moves(), key=root.visit_count, reverse=True)[:5]
                top_5 = [f"{game_state.san(x)} {root.visit_count(x)}" for x in top_5]
                t.set_description(" | ".join(top_5))

            # Return early if a branch is guaranteed
            if root.check_visit_counts(self.num_rounds):
                break

        if self.verbose:
            run_time = timeit.default_timer() - start_time - self.eval_time
            print(" | ".join(top_5))
            print(f"Eval time: {self.eval_time:.2f} | Calc time: {run_time:.2f} |", end=" ")
            print(f"{self.eval_time / (self.eval_time + run_time) * 100:.2f}%")

        return max(root.moves(), key=root.visit_count)
