Bitvec index is position
However its not really possible to store positions out of bounds this way
BUUUT you could use negative indices and indices outside of bounds of the array
BUUUT these require significantly larger data structures

usize is large, use smaller index primitives where possible

Seekers come at you if you get to close
bombers blow up if you get too close

World collisions are bad, only need one per ent

