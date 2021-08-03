import json
import os
import threading
import time
from pathlib import Path
from timeit import default_timer as timer

import berserk
import chess
import chess.polyglot
from dotenv import load_dotenv

from botfjord import mcts_rust
from botfjord.types import Limit


class Game(threading.Thread):
    def __init__(self, client: berserk.Client, game_id: str):
        self._is_running = True
        super().__init__()

        self.game_id = game_id
        self.client = client
        self.stream = self.client.bots.stream_game_state(game_id)

        self.board = None
        self.get_game_state()

        self._is_searching = False
        self._opp_timer = timer()
        self.limit = Limit(time=1)

        self.name = client.account.get()["id"]
        self._chat_active = True
        self.white = self.name if self.color == "white" else self.opponent["username"]
        self.black = self.name if self.color == "black" else self.opponent["username"]
        print(f"Game {self.game_id} | Initialized | {self.white} v {self.black}")

    def run(self):
        while self._is_running:
            for event in self.stream:
                if event["type"] == "gameFull":
                    self.initial_fen = (
                        event["initialFen"]
                        if event["initialFen"] != "startpos"
                        else chess.STARTING_FEN
                    )
                    self.board = chess.Board(self.initial_fen)
                    event = event["state"]
                    self.get_game_state()

                if event["type"] == "gameState":
                    self.set_time(event)
                    self._is_searching = True
                    move_list = event["moves"].split()
                    if (self.color == "white") ^ len(move_list) % 2 == 0:
                        continue
                    else:
                        self.board = chess.Board(self.initial_fen)
                        for uci_move in move_list:
                            self.board.push_uci(uci_move)
                    self._is_searching = False

                if event["type"] == "chatLine":
                    self.handle_chat()

                self.check_turn()

    def stop(self):
        print(f"Game {self.game_id} | Exited")
        self._is_running = False

    def get_game_state(self, ongoing: list = None):
        if ongoing is None:
            ongoing = self.client.games.get_ongoing(count=10)
        for game in ongoing:
            if game["gameId"] == self.game_id:
                self.game = game
                self.fen = self.game["fen"]
                self.color = self.game["color"]
                self.op_color = "white" if self.color == "black" else "black"
                self.opponent = self.game["opponent"]
                self.is_my_turn = self.game["isMyTurn"]
                self.last_move = self.game["lastMove"]
                return
        self.stop()  # Exit if none

    def update_game_state(self):
        self.is_my_turn = (self.color == "white") == (len(self.board.move_stack) % 2 == 0)

    def check_turn(self):
        """Check if currently our turn and make a move if it is"""
        self.update_game_state()
        if self.is_my_turn and not self._is_searching and not self.board.is_game_over():
            self._is_searching = True
            self.calculate_limit()
            next_move = get_move(self.board, self.limit)
            try:
                client.bots.make_move(game_id=self.game_id, move=next_move)
                self.board.push_uci(next_move)
                self._opp_timer = timer()
            except:
                self._is_searching = False
                return
            self.update_game_state()
            self._is_searching = False

    def handle_chat(self):
        if self._chat_active:
            client.bots.post_message(
                game_id=self.game_id, text="Sorry, I'm not set up for chat yet!"
            )
            self._chat_active = False
        self.get_game_state()

    def set_time(self, event):
        """Updates time and increment values"""
        try:
            self.wtime = (
                event["wtime"].second
                + (event["wtime"].hour * 3600)
                + (event["wtime"].minute * 60)
                + (event["wtime"].microsecond / 1000000)
            )
            self.btime = (
                event["btime"].second
                + (event["btime"].hour * 3600)
                + (event["btime"].minute * 60)
                + (event["btime"].microsecond / 1000000)
            )
            self.winc = event["winc"].second + event["winc"].minute * 60
            self.binc = event["binc"].second + event["binc"].minute * 60
        except AttributeError:
            self.wtime = event["wtime"] / 1000
            self.btime = event["btime"] / 1000
            self.winc = event["winc"] / 1000
            self.binc = event["binc"] / 1000

    def calculate_limit(self):
        """Calculate and set time limit based on remaining time and increment"""
        opp_time = timer() - self._opp_timer
        rem_time = self.wtime if self.color == "white" else self.btime
        rem_time -= self.limit.time  # Account for search time by subtracting previous time

        inc = self.winc if self.color == "white" else self.binc
        inc *= 0.9

        rem_moves_50 = 50 - self.board.fullmove_number  # 50 move avg game length
        rem_moves_100 = 100 - self.board.fullmove_number  # 100 move long game length
        if rem_moves_50 > 0:
            limit = (((rem_time * 0.5) / rem_moves_50) * 0.75) + (
                ((rem_time * 0.9) / rem_moves_100) * 0.25
            )
            if rem_moves_50 < 10:
                limit = min(limit / ((50 - rem_moves_50) / 50 * 2), limit)
        elif rem_moves_100 > 0:
            limit = (rem_time * 0.5) / rem_moves_100
            limit = min(limit / ((100 - rem_moves_100) / 100 * 2), limit)
        else:
            limit = rem_time * 0.1

        limit += inc
        # Average with opponent's last time to avoid long turns against human players
        avg_limit = (limit + opp_time) / 2
        limit = min(limit, rem_time, avg_limit, 10)
        limit = max(limit - 0.4, 0.01)
        self.limit = Limit(time=limit)


def get_move(game_state: chess.Board, limit: Limit) -> str:
    """Handles getting move from search or opening book"""
    if _book:
        with chess.polyglot.open_reader(os.environ["OPENING_BOOK_PATH"]) as reader:
            try:
                move = reader.weighted_choice(game_state).move.uci()
                time.sleep(0.1)
                return move
            except IndexError:
                ...

    return mcts_rust.search_tree(game_state.fen(), limit.time, temperature, processes)


def auto_check():
    """Automatic timer to check game status preventing stuck games"""
    ongoing = client.games.get_ongoing(count=10)

    for game_info in ongoing:
        if game_info["gameId"] not in [game.game_id for game in games]:
            game = Game(client=client, game_id=game_info["gameId"])
            print(f"Game {game.game_id} | Force Start")
            games.append(game)
            game.start()
            time.sleep(0.1)

    for game in games:
        game.get_game_state(ongoing)

    t = threading.Timer(5, auto_check)
    t.start()


def should_accept(event):
    """Returns bool for if game should be accepted based on configured parameters"""
    if (
        (event["challenger"]["id"].lower() in accept_players or accept_players == [])
        and (event["speed"] in accept_timecontrol or accept_timecontrol == [])
        and event["variant"]["key"] in ["standard", "fromPosition"]
        and len(games) <= max_games
    ):
        return True
    else:
        return False


if __name__ == "__main__":
    load_dotenv()

    processes = int(os.environ["SEARCH_PROCESSES"])
    temperature = float(os.environ["SEARCH_TEMPERATURE"])
    accept_players = json.loads(os.environ["ACCEPT_PLAYERS"])
    accept_timecontrol = json.loads(os.environ["ACCEPT_TIMECONTROL"])
    max_games = int(os.environ["MAX_GAMES"])

    _book = True
    if not Path(os.environ["OPENING_BOOK_PATH"]).exists():
        print("Invalid opening book path | Opening book disabled")
        _book = False

    session = berserk.TokenSession(os.environ["LICHESS_TOKEN"])
    client = berserk.Client(session)

    games: list[Game] = []

    for event in client.bots.stream_incoming_events():
        if event["type"] == "challenge":
            if should_accept(event["challenge"]):
                try:
                    client.bots.accept_challenge(event["challenge"]["id"])
                except:
                    continue
            else:
                try:
                    client.bots.decline_challenge(event["challenge"]["id"])
                except:
                    continue
        elif event["type"] == "gameStart":
            if event["game"]["id"] not in [game.game_id for game in games]:
                game = Game(client=client, game_id=event["game"]["id"])
                print(f"Game {game.game_id} | Start")
                games.append(game)
                game.start()
                time.sleep(0.1)

    auto_check()
