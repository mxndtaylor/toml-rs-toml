use std::ops::Index;

struct Position {
    undotted: usize,
    dotted: dyn Index<&str, Output = Position>,
}


