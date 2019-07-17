//! Testing how to create a shared global state over all spawned
//! tasks.
//!
//! Key here is that the shared state should be available for all
//! spawned threads with a lifetime that spans them.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;

#[derive(Debug)]
struct State {
    version: i32,
}

impl State {
    fn new() -> Self {
        Self { version: 0 }
    }

    fn inc(&mut self) {
        self.version += 1;
    }

    fn dec(&mut self) {
        self.version -= 1;
    }
}

fn main() {
    tokio::run(futures::lazy(|| {
        let shared_state = Arc::new(Mutex::new(State::new()));

        let first = {
            let state = shared_state.clone();
            Interval::new(Instant::now(), Duration::from_millis(5000))
                .for_each(move |instant| {
                    let mut locked_state = state.lock().unwrap();
                    locked_state.dec();
                    println!("first - instant={:?}, state={:?}", instant, locked_state);
                    Ok(())
                })
                .map_err(|e| panic!("first - interval errored; err={:?}", e))
        };

        tokio::spawn(first);

        let second = {
            let state = shared_state.clone();
            Interval::new(Instant::now(), Duration::from_millis(500))
                .for_each(move |instant| {
                    let mut locked_state = state.lock().unwrap();
                    locked_state.inc();
                    println!("second - instant={:?}, state={:?}", instant, locked_state);
                    Ok(())
                })
                .map_err(|e| panic!("second - interval errored; err={:?}", e))
        };

        tokio::spawn(second);
        Ok(())
    }));
}
