use std::{io::{Error, Read}, path::Path};

static NUMBER_OF_TRHEADS_MASK: i16  = 0x7FFF;
static NUMBER_OF_LOCKS_MASK: i32    = 0x7FFFFFFF;
static NUMBER_OF_VARS_MASK: i32     = 0x7FFFFFFF;
static NUMBER_OF_EVENTS_MASK: i64   = 0x7FFFFFFFFFFFFFFF;


static THREAD_NUM_BITS: u16 = 10;
static THREAD_BIT_OFFSET: u16 = 0;

static OP_NUM_BITS: u16 = 4;
static OP_BIT_OFFSET: u16 = THREAD_NUM_BITS;

static DECOR_NUM_BITS: u16 = 34;
static DECOR_BIT_OFFSET: u16 = THREAD_NUM_BITS + OP_NUM_BITS;

static LOC_NUM_BITS: u16 = 15;
static LOC_BIT_OFFSET: u16 = THREAD_NUM_BITS + OP_NUM_BITS + DECOR_NUM_BITS;

static THREAD_MASK: i64 = ((1 << THREAD_NUM_BITS) - 1) << THREAD_BIT_OFFSET;
static OP_MASK: i64 = ((1 << OP_NUM_BITS) - 1) << OP_BIT_OFFSET;
static DECOR_MASK: i64 = ((1 << DECOR_NUM_BITS) - 1) << DECOR_BIT_OFFSET;
static LOC_MASK: i64 = ((1 << LOC_NUM_BITS) - 1) << LOC_BIT_OFFSET;

pub struct Event {
    t: i64,
    op: i64,
    decor: i64,
    loc: i64,
}

impl Event {
    fn get_formatted_decor(&self) -> String {
        match self.op {
            2 => format!("r(V{})", self.decor),
            3 => format!("w(V{})", self.decor),
            0 => format!("acq(L{})", self.decor),
            8 => format!("req(L{})", self.decor),
            1 => format!("rel(L{})", self.decor),
            4 => format!("fork(T{})", self.decor),
            5 => format!("join(T{})", self.decor),
            _ => panic!("Unknown operator {}!", self.op)
        }
    }

    fn to_std_format(&self) -> String {
        format!("T{}|{}|{}", self.t, self.get_formatted_decor(),self.loc)
    }
}

pub fn parse_from_file(path: &Path) -> Result<Vec<Event>, Error> {
    Ok(parse_from_buffer(&std::fs::read(path)?))
}

pub fn parse_from_buffer(mut buffer: &[u8]) -> Vec<Event> {
    let mut out = Vec::new();
    
    let mut n_threads = [0; 2];
    assert_eq!(buffer.read(&mut n_threads).expect("Could not read number of threads from the buffer!"), 2);
    let n_threads = NUMBER_OF_TRHEADS_MASK & i16::from_be_bytes(n_threads);
    println!("Number of Threads:\t{}", n_threads);
    
    let mut n_locks = [0; 4];
    assert_eq!(buffer.read(&mut n_locks).expect("Could not read number of locks from the buffer!"), 4);
    let n_locks = NUMBER_OF_LOCKS_MASK & i32::from_be_bytes(n_locks);
    println!("Number of Locks:\t{}", n_locks);

    let mut n_vars = [0; 4];
    assert_eq!(buffer.read(&mut n_vars).expect("Could not read number of variables from the buffer!"), 4);
    let n_vars = NUMBER_OF_VARS_MASK & i32::from_be_bytes(n_vars);
    println!("Number of Variables:\t{}", n_vars);

    let mut n_events = [0; 8];
    assert_eq!(buffer.read(&mut n_events).expect("Could not read number of variables from the buffer!"), 8);
    let n_events = NUMBER_OF_EVENTS_MASK & i64::from_be_bytes(n_events);
    println!("Number of Events:\t{}", n_events);

    let mut event_buffer = [0; 8];
    for i in 0..n_events {
        assert_eq!(buffer.read(&mut event_buffer).unwrap_or_else(|_| panic!("Could not {}th event from the buffer!", i)), 8);
        let event_integer = i64::from_be_bytes(event_buffer);
        
        let t = (event_integer & THREAD_MASK) >> THREAD_BIT_OFFSET;
        let op = (event_integer & OP_MASK) >> OP_BIT_OFFSET;
        let decor = (event_integer & DECOR_MASK) >> DECOR_BIT_OFFSET;
        let loc = (event_integer & LOC_MASK) >> LOC_BIT_OFFSET;
        let event = Event { t, op, decor, loc };

        println!("Parsed Event: {}", event.to_std_format());

        out.push(event);
    }

    out
}

pub fn emit_text_format(trace: Vec<Event>) -> Vec<String> {
    trace.into_iter().map(|e| e.to_std_format()).collect()
}

pub fn convert_bin_to_text(buffer: &[u8]) -> Vec<String> {
    emit_text_format(parse_from_buffer(buffer))
}