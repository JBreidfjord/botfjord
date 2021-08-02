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
