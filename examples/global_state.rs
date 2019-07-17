// Copyright 2019 Mats Kindahl
//
// Licensed under the Apache License, Version 2.0 (the "License"); you
// may not use this file except in compliance with the License.  You
// may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.  See the License for the specific language governing
// permissions and limitations under the License.

// Testing how to create a shared global state over all spawned
// tasks.
//
// Key here is that the shared state should be available for all
// spawned threads with a lifetime that spans them, but not have to
// have a static lifetime.

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
    // We create a lazy future that create the state to be shared and
    // pass that to `tokio::run`.
    //
    // We then crate two independent interval streams that will
    // increase and decrease the version of the shared state at
    // different paces.
    tokio::run(futures::lazy(|| {
        let shared_state = Arc::new(Mutex::new(State::new()));

        tokio::spawn({
            let state = shared_state.clone();
            Interval::new(Instant::now(), Duration::from_millis(5000))
                .for_each(move |instant| {
                    let mut locked_state = state.lock().unwrap();
                    locked_state.dec();
                    println!("first - instant={:?}, state={:?}", instant, locked_state);
                    Ok(())
                })
                .map_err(|e| panic!("first - interval errored; err={:?}", e))
        });

        tokio::spawn({
            let state = shared_state.clone();
            Interval::new(Instant::now(), Duration::from_millis(500))
                .for_each(move |instant| {
                    let mut locked_state = state.lock().unwrap();
                    locked_state.inc();
                    println!("second - instant={:?}, state={:?}", instant, locked_state);
                    Ok(())
                })
                .map_err(|e| panic!("second - interval errored; err={:?}", e))
        });

        Ok(())
    }));
}
