extern crate page_size;
extern crate lazy_static;

use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};
use std::thread;
use std::time::{Instant, Duration};
use std::sync::mpsc::{Receiver, channel};
use lazy_static::lazy_static;
use systemstat::{System, Platform};

mod heartbeat;
mod controller;
use controller::Controller;

fn timer(delay: Duration) -> Receiver<()> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        loop {
            if let Err(_) = tx.send(()) {
                return;
            }
            thread::sleep(delay);
        }
    });
    rx
}

lazy_static! {
    static ref HB_INTVL: Duration = Duration::from_millis(100);
    static ref MEASURE_INTVL: Duration = Duration::from_millis(100);
}

const HB_INC: u64 = 10;
const ITERS: u64 = 600;
const MAX_MEM_USAGE: f64 = 6_000_000_000.0;

//Taken directly from code paste
fn main() {
    let mut h = heartbeat::Heartbeat::default();
    let mut cont = Controller::new(page_size::get());

    let hb_timer = timer(*HB_INTVL);

    let done_tx = Arc::new(AtomicBool::new(false));
    let done_rx = done_tx.clone();
    let hb_worker = thread::spawn(move || {
        for _ in 0..ITERS {
            hb_timer.recv().unwrap();
            cont.adjust((h.level() * MAX_MEM_USAGE) as _);
            //println!("{}\t{}", i, h.level());
            h.inc_time(HB_INC);
        }

        done_tx.store(true, Ordering::Release);
    });
    
    let measure_timer = timer(*MEASURE_INTVL);
    
    let sys = System::new();
    let start = Instant::now();
    while !done_rx.load(Ordering::Relaxed) {
        measure_timer.recv().unwrap();
        let mem = sys.memory().unwrap();
        let millis = (start.elapsed().as_nanos() as f64) / 1_000_000.0;
        println!("{}\t{}", millis, (mem.total - mem.free).as_usize() >> 20);
    }

    hb_worker.join().unwrap();
}
