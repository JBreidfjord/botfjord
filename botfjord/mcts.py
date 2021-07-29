from __future__ import annotations

import timeit
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass
from math import log, sqrt

import numpy as np
from chess import Board, Move


@dataclass
class Branch:
    prior: float
    visit_count: int = 0
    total_value: float = 0.0


class Limit:
    def __init__(self, time: float = None, nodes: int = None):
        """
        Search termination condition. Defaults to 3200 nodes if no args given.\n
        Args:
            time: seconds as float
            nodes: search rounds as int
        """
        self.time = time
        self.nodes = nodes
        if self.time is None and self.nodes is None:
            nodes = 3200


class Node:
    def __init__(
        self,
        state: Board,
        value: float,
        priors: dict[Move, float],
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
        branches = sorted(self.branches.values(), key=lambda b: b.visit_count)
        remaining_rounds = num_rounds - self.total_visit_count
        return branches[-1].visit_count >= branches[-2].visit_count + remaining_rounds

    def check_visit_ratio(self, factor: float = 0.75, minimum: int = 100):
        """Returns True if the most visited branch has been visited more than
        the total visit count * factor and total visit count is above minimum"""
        if self.total_visit_count < minimum:
            return False
        branch = max(self.branches.values(), key=lambda b: b.visit_count)
        return branch.visit_count > self.total_visit_count * factor


class MCTS:
    def __init__(
        self,
        evaluation_function: function,
        prior_function: function,
        temperature: float = sqrt(2),
        noise: float = 0.3,
    ):
        """
        Monte Carlo Tree Search\n
        Args:
            evaluation_function & prior_function:
                The functions to calculate the value of game states.\n\t
                Prior function should be a small, fast calculation 
                used to quickly determine branch priority without searching.\n\t
                Function should return a dict[Move, float] probability distribution
                over all legal moves.\n\t
                Evaluation function is the main function used for
                calculating the value of a state.\n\t
                Function should return float.\n
            temperature: Constant value that determines the exploration/exploitation
            trade-off when searching.\n
            noise: Alpha value for Dirichlet noise
        """
        self._eval_func = evaluation_function
        self._prior_func = prior_function
        self._c = temperature
        self.noise = noise
        self._rng = np.random.default_rng()

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
        priors = self._prior_func(game_state)
        value = self._eval_func(game_state)

        # Add Dirichlet noise
        if self.noise is not None:
            noise = self._rng.dirichlet([self.noise] * game_state.legal_moves.count())
            for (mv, val), noise_val in zip(priors.items(), noise):
                priors[mv] = (0.5 * val) + (0.5 * noise_val)

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
            return q + self._c * p * (sqrt(log(total_n) / (n + 1e-7)))

        return max(node.moves(), key=score_branch)

    def search(self, game_state: Board, limit: Limit = Limit(nodes=3200)):
        """The core of the MCTS class.
        Starts a search from the given Board and returns the selected Move."""

        _active = True
        i = 0
        start_time = timeit.default_timer()

        root = self.create_node(game_state)
        while _active:
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

            if root.check_visit_ratio():
                _active = False

            if limit.nodes is not None:
                if i >= limit.nodes or root.check_visit_counts(limit.nodes):
                    _active = False
                else:
                    i += 1
            if limit.time is not None:
                if timeit.default_timer() - start_time >= limit.time:
                    _active = False

        return [(move, root.visit_count(move)) for move in root.moves()]

    def _params(self):
        return {
            "evaluation_function": self._eval_func,
            "prior_function": self._prior_func,
            "temperature": self._c,
            "noise": self.noise,
        }


def worker(agent_params: dict, limit_params: dict, fen: str):
    agent = MCTS(**agent_params)
    limit = Limit(**limit_params)
    game_state = Board(fen)
    result = agent.search(game_state, limit)
    move_dict = {}
    for move, visit_count in result:
        move_dict[move] = visit_count
    return move_dict


def mp_search(
    agent: MCTS, game_state: Board, limit: Limit, processes: int = 16, verbose: bool = True
):
    # Return early if only 1 legal move available
    if game_state.legal_moves.count() == 1:
        return next(game_state.generate_legal_moves())
    fen = game_state.fen()
    agent_params = agent._params()
    limit_params = {"time": limit.time, "nodes": limit.nodes}

    move_dict = {move: 0 for move in game_state.legal_moves}
    start_time = timeit.default_timer()
    with ProcessPoolExecutor() as pool:
        futures = [pool.submit(worker, agent_params, limit_params, fen) for _ in range(processes)]
        for future in futures:
            result = future.result()
            for move in game_state.legal_moves:
                move_dict[move] += result[move]

    if verbose:
        top_5 = sorted(move_dict.items(), key=lambda x: x[1], reverse=True)[:5]
        top_5 = [f"{game_state.san(mv)} {vc}" for mv, vc in top_5]
        run_time = timeit.default_timer() - start_time
        nodes = sum(move_dict.values())
        print(
            " | ".join(top_5)
            + f" | {round(nodes / run_time)} nodes/s ({run_time:.2f}s | {nodes} nodes)"
        )

    return max(move_dict.items(), key=lambda x: x[1])[0]
