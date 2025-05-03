enum Op {
    Read { addr: usize, n: usize },
    Write { addr: usize, n: usize },
    Aquire { lock: usize },
    Request { lock: usize },
    Release { lock: usize },
    Fork { tid: usize },
    Join { tid: usize },
}

pub struct Event {
    t: usize,            // ID of the executing thread
    op: Op,              // executed operation
    loc: (usize, usize), // location in the program: (function_idx, instr_idx)
}
