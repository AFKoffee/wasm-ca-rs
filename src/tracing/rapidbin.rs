use std::collections::HashMap;

use super::Event;

/*
Only relevant for reading traces:

static NUMBER_OF_TRHEADS_MASK: i16  = 0x7FFF;
static NUMBER_OF_LOCKS_MASK: i32    = 0x7FFFFFFF;
static NUMBER_OF_VARS_MASK: i32     = 0x7FFFFFFF;
static NUMBER_OF_EVENTS_MASK: i64   = 0x7FFFFFFFFFFFFFFF;
*/

static THREAD_NUM_BITS: u16 = 10;
static THREAD_BIT_OFFSET: u16 = 0;

static OP_NUM_BITS: u16 = 4;
static OP_BIT_OFFSET: u16 = THREAD_NUM_BITS;

static DECOR_NUM_BITS: u16 = 34;
static DECOR_BIT_OFFSET: u16 = THREAD_NUM_BITS + OP_NUM_BITS;

static LOC_NUM_BITS: u16 = 15;
static LOC_BIT_OFFSET: u16 = THREAD_NUM_BITS + OP_NUM_BITS + DECOR_NUM_BITS;

/*
Only relevant for reading traces:

static THREAD_MASK: i64 = ((1 << THREAD_NUM_BITS) - 1) << THREAD_BIT_OFFSET;
static OP_MASK: i64 = ((1 << OP_NUM_BITS) - 1) << OP_BIT_OFFSET;
static DECOR_MASK: i64 = ((1 << DECOR_NUM_BITS) - 1) << DECOR_BIT_OFFSET;
static LOC_MASK: i64 = ((1 << LOC_NUM_BITS) - 1) << LOC_BIT_OFFSET;
*/

pub struct BinaryTraceBuilder {
    thread_map: HashMap<u32, i16>,
    thread_counter: i16,
    memory_map: HashMap<(usize, usize), i32>,
    memory_counter: i32,
    lock_map: HashMap<usize, i32>,
    lock_counter: i32,
    location_map: HashMap<(usize, usize), i16>,
    location_counter: i16,
    binary_trace: Vec<i64>,
    event_counter: i64,
}

impl BinaryTraceBuilder {
    pub fn new() -> Self {
        Self { 
            thread_map: HashMap::new(), 
            thread_counter: 0, 
            memory_map: HashMap::new(), 
            memory_counter: 0, 
            lock_map: HashMap::new(), 
            lock_counter: 0, 
            location_map: HashMap::new(),
            location_counter: 0,
            binary_trace: Vec::new(),
            event_counter: 0,
        }
    }

    fn get_thread_identifier(&mut self, t: &u32) -> i16 {
        if let Some(tid) = self.thread_map.get(t) {
            *tid
        } else {
            let tid = self.thread_counter;
            self.thread_map.insert(*t, tid);
            self.thread_counter += 1;
            tid
        }
    }

    fn get_location_identifier(&mut self, loc: &(usize, usize)) -> i16 {
        if let Some(loc_id) = self.location_map.get(loc) {
            *loc_id
        } else {
            let loc_id = self.location_counter;
            self.location_map.insert(*loc, loc_id);
            self.location_counter += 1;
            loc_id
        }
    }

    fn get_memory_identifier(&mut self, addr: &usize, n: &usize) -> i32 {
        if let Some(mem_id) = self.memory_map.get(&(*addr, *n)) {
            *mem_id
        } else {
            let mem_id = self.memory_counter;
            self.memory_map.insert((*addr, *n), mem_id);
            self.memory_counter += 1;
            mem_id
        }
    }

    fn get_lock_identifier(&mut self, lock: &usize) -> i32 {
        if let Some(lock_id) = self.lock_map.get(lock) {
            *lock_id
        } else {
            let lock_id = self.lock_counter;
            self.lock_map.insert(*lock, lock_id);
            self.lock_counter += 1;
            lock_id
        }
    }

    fn convert_event(&mut self, event: &Event) -> i64 {
        let Event{t, op, loc} = event;

        // TODO: Look at this part again:
        // We clip the values of the event elements here to ensure they have at maximum their
        // designated number of bits set for the OR operation later on. 
        // However, the program currently does not check, whether the value was in the valid
        // range or not possibly resulting in silent errors (invalid traces).
        let thread_id = i64::from(self.get_thread_identifier(t)) & ((1 << THREAD_NUM_BITS) - 1);
        let op_id = i64::from(op.id()) & ((1 << OP_NUM_BITS) - 1);    
        let location_id = i64::from(self.get_location_identifier(loc)) & ((1 << LOC_NUM_BITS) - 1);
        let decor = i64::from(match op {
            super::Op::Read { addr, n } |
            super::Op::Write { addr, n } => self.get_memory_identifier(addr, n),
            super::Op::Aquire { lock } |
            super::Op::Request { lock } |
            super::Op::Release { lock } => self.get_lock_identifier(lock),
            super::Op::Fork { tid } |
            super::Op::Join { tid } => i32::from(self.get_thread_identifier(tid)),
        }) & ((1 << DECOR_NUM_BITS) - 1);

        (thread_id << THREAD_BIT_OFFSET) |
            (op_id << OP_BIT_OFFSET) |
            (decor << DECOR_BIT_OFFSET) |
            (location_id << LOC_BIT_OFFSET)
    }

    pub fn push_event(&mut self, event: &Event) {
        let binary_event = self.convert_event(event);
        self.binary_trace.push(binary_event);
        self.event_counter += 1;
    }

    pub fn build(self) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.binary_trace.len() * std::mem::size_of::<i64>() + 12);

        output.extend(self.thread_counter.to_be_bytes());
        output.extend(self.lock_counter.to_be_bytes());
        output.extend(self.memory_counter.to_be_bytes());
        output.extend(self.event_counter.to_be_bytes());
        output.extend(self.binary_trace.into_iter().flat_map(|e| e.to_be_bytes()));
        
        output
    }
}

#[cfg(test)]
mod test {
    use crate::tracing::{Event, Op};

    use super::{BinaryTraceBuilder, THREAD_BIT_OFFSET, OP_BIT_OFFSET, DECOR_BIT_OFFSET, LOC_BIT_OFFSET};

    #[test]
    fn test_event_conversion() {
        let mut builder = BinaryTraceBuilder::new();
        let event = Event {t: 1, op: Op::Write { addr: 100, n: 2 }, loc: (10, 75)};
        let binary_event = (0 << THREAD_BIT_OFFSET) |
            (3 << OP_BIT_OFFSET) |
            (0 << DECOR_BIT_OFFSET) |
            (0 << LOC_BIT_OFFSET);
        assert_eq!(builder.convert_event(&event), binary_event)
    }
}