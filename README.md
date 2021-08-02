# Botfjord
A chess engine powered by Rust, running on Lichess with Python. You can play against it [here](https://lichess.org/@/Botfjord) if it is currently online.

### Search Algorithm
It uses [Monte Carlo tree search](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search) as the core search algorithm.
Instead of performing playouts at leaf nodes like a Pure MCTS algorithm would, it uses an evaluation function to give a value to the node.
In addition to this value, it will also calculate a prior value for all possible child nodes of the current node.
It uses these values in a modified [UCT formula](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation):

![equation](http://www.sciweavers.org/tex2img.php?eq=q%20%2B%20p%20%5Ctimes%20c%20%5Ctimes%20%5Csqrt%7B%5Cfrac%7Bln%28N%29%7D%7Bn%7D%7D%20&bc=White&fc=Black&im=png&fs=24&ff=arev&edit=0)

Where:

 - q is the expected value for the current node, calculated as the average value of itself and its explored children,
  
 - p is the prior value for the current node,
  
 - c is a constant search temperature to affect the exploration/exploitation trade-off,
  
 - N is the total visit count of the parent node,
  
 - n is the visit count of the current node.
 
When n = 0, the second term of the equation is replaced by an arbitrarily high value so all branches will be explored once at a minimum.

The next branch to search is calculated by finding the unexplored leaf node that maximizes the output of the formula.

The current evaluation function is a simple piece value calculation with a few minor modifications.
The current prior evaluation function is an even simpler difference in number of pieces.
