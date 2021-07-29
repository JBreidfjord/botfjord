from chess import Board, Move, SquareSet

piece_value_map = {1: 1, 2: 3, 3: 3, 4: 5, 5: 9, 6: 0.0}


def eval_fn(board: Board):
    value = 0.0
    if board.is_checkmate():
        return -39
    if board.is_repetition(2):
        return -39 if board.turn else 39

    # Gets number of pieces per type per side then adds to value
    black, white = board.occupied_co
    value -= len(SquareSet(black & board.pawns)) * piece_value_map[1]
    value -= len(SquareSet(black & board.knights)) * piece_value_map[2]
    value -= len(SquareSet(black & board.bishops)) * piece_value_map[3]
    value -= len(SquareSet(black & board.rooks)) * piece_value_map[4]
    value -= len(SquareSet(black & board.queens)) * piece_value_map[5]
    value += len(SquareSet(white & board.pawns)) * piece_value_map[1]
    value += len(SquareSet(white & board.knights)) * piece_value_map[2]
    value += len(SquareSet(white & board.bishops)) * piece_value_map[3]
    value += len(SquareSet(white & board.rooks)) * piece_value_map[4]
    value += len(SquareSet(white & board.queens)) * piece_value_map[5]

    if not board.turn:
        value *= -1

    if board.is_check():
        value -= 0.75

    return value


def prior_fn(board: Board):
    priors: dict[Move, float] = {}

    def score(board: Board):
        value = 0.0
        if board.is_checkmate():
            return -39

        # Gets number of pieces per type per side then adds to value
        black, white = board.occupied_co
        value -= len(SquareSet(black & board.pawns)) * piece_value_map[1]
        value -= len(SquareSet(black & board.knights)) * piece_value_map[2]
        value -= len(SquareSet(black & board.bishops)) * piece_value_map[3]
        value -= len(SquareSet(black & board.rooks)) * piece_value_map[4]
        value -= len(SquareSet(black & board.queens)) * piece_value_map[5]
        value += len(SquareSet(white & board.pawns)) * piece_value_map[1]
        value += len(SquareSet(white & board.knights)) * piece_value_map[2]
        value += len(SquareSet(white & board.bishops)) * piece_value_map[3]
        value += len(SquareSet(white & board.rooks)) * piece_value_map[4]
        value += len(SquareSet(white & board.queens)) * piece_value_map[5]

        if not board.turn:
            value *= -1

        if board.is_check():
            value -= 0.75

        return value

    for move in board.legal_moves:
        board.push(move)
        priors[move] = score(board) + 1e-7
        board.pop()

    if priors == {}:
        return priors

    # Remove negatives and invert values
    abs_min = abs(min(priors.values()))
    max_ = max(priors.values()) + abs_min
    max_ *= 1.25
    for move in priors:
        priors[move] = max_ - (priors[move] + abs_min)

    norm_factor = 1.0 / sum(priors.values(), start=1e-7)
    for move in priors:
        priors[move] *= norm_factor

    return priors
