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
//
// For the example, we create two independent interval streams that
// will increase and decrease the version of the shared state at
// different paces.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::stream::StreamExt;
use tokio::time::interval;

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

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(State::new()));

    // Note that we are here first creating a block where we clone the
    // Arc of the shared state and then pass that into the async
    // block. If we didn't do that, the shared_state would be borrowed
    // inside the async block and this block can outlive the
    // shared_state *variable* (not the underlying state).
    let handle1 = tokio::spawn({
        let state = shared_state.clone();
        async move {
            let mut ticker = interval(Duration::from_millis(5000));
            while let Some(instant) = ticker.next().await {
                let mut locked_state = state.lock().unwrap();
                locked_state.dec();
                println!("first - instant={:?}, state={:?}", instant, locked_state);
            }
        }
    });

    let handle2 = tokio::spawn({
        let state = shared_state.clone();
        async move {
            let mut ticker = interval(Duration::from_millis(500));
            while let Some(instant) = ticker.next().await {
                let mut locked_state = state.lock().unwrap();
                locked_state.inc();
                println!("second - instant={:?}, state={:?}", instant, locked_state);
            }
        }
    });

    println!("{:?}", handle1.await);
    println!("{:?}", handle2.await);
}
