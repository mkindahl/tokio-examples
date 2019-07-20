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

// Example to demonstrate how to create a stream that can be halted
// and resumed. The key problem here is that you want to return
// `Async::NotReady` and still have a guarantee that an attempt to
// call it will be done later.
//
// This was based on the problem of creating an infinite stream over a
// cycled list, but if the list is empty, the stream should not
// produce anything until the list is actually non-empty.

use futures::stream::Stream;
use futures::task::{self, Task};
use futures::{Async, Poll};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::timer::Interval;

// Cyclic stream.
//
// This is a custom stream that will return the items in a vector in
// cyclic order. The vector is mutex-protected and can be changed by
// some other thread.
#[derive(Debug)]
struct MyStream {
    state: Arc<Mutex<State>>,
    index: usize,
}

impl MyStream {
    fn new(state: Arc<Mutex<State>>) -> MyStream {
        MyStream {
            state: state,
            index: 0,
        }
    }
}

impl Stream for MyStream {
    type Item = i32;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut locked_state = self.state.lock().unwrap();
        if locked_state.array.len() > 0 {
            // If the array contains something, just return the next
            // items in the vector in a cyclic fashion.
            if self.index >= locked_state.array.len() {
                self.index = 0;
            }
            let result = locked_state.array[self.index];
            self.index += 1;
            Ok(Async::Ready(Some(result)))
        } else {
            // If the array is empty, just ask to be notified in the
            // future and say that the stream is not ready.
            //
            // If we do not ask to be notified in the future
            // explicitly, this future will never be scheduled again.
            locked_state.task = Some(task::current());
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
struct State {
    array: Vec<i32>,
    task: Option<Task>,
}

impl State {
    fn new() -> Self {
        Self {
            array: Vec::new(),
            task: None,
        }
    }
}

fn main() {
    let shared_state = Arc::new(Mutex::new(State::new()));

    // This future just produces one number each second from the
    // stream.
    let numbers = MyStream::new(shared_state.clone())
        .zip(
            Interval::new(Instant::now(), Duration::from_millis(1000))
                .map_err(|err| println!("Error: {}", err)),
        )
        .for_each(|(number, _instant)| {
            println!("got number {}", number);
            Ok(())
        });

    // This future run every 5 seconds and insert an item into the
    // array, up to a limit of 5, and then clears the array again.
    let on_off = {
        let state = shared_state.clone();
        // This variable will retain the state between invocations of
        // the closure below.
        let mut val = 0;
        Interval::new(Instant::now(), Duration::from_millis(5000))
            .map_err(|err| println!("Error: {}", err))
            .for_each(move |_instant| {
                let mut locked_state = state.lock().unwrap();
                if locked_state.array.len() < 5 {
                    println!("pushing {} on array", val);
                    locked_state.array.push(val);
                    val += 1;
                    if let Some(task) = locked_state.task.take() {
                        task.notify();
                    }
                } else {
                    println!("clearing array");
                    locked_state.array.clear();
                }
                Ok(())
            })
    };

    tokio::run(futures::lazy(|| {
        tokio::spawn(numbers);
        tokio::spawn(on_off);
        Ok(())
    }));
}
