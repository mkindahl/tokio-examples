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
// `Async::NotReady` and stillhave a guarantee that an attempt to
// call it will be done later.
//
// This was based on the problem of creating an infinite stream over a
// cycled list, but if the list is empty, the stream should not
// produce anything until the list is actually non-empty.

use futures::stream::Stream;
use futures::{future, StreamExt};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::interval;

// Cyclic stream.
//
// This is a custom stream that will return the items in a vector in
// cyclic order. The vector is mutex-protected and can be changed by
// some other thread.
#[derive(Debug)]
struct MyStream {
    state: Arc<Mutex<State>>,
}

impl MyStream {
    fn new(state: Arc<Mutex<State>>) -> MyStream {
        MyStream { state: state }
    }
}

impl Stream for MyStream {
    type Item = Result<i32, ()>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut locked_state = self.state.lock().unwrap();
        if locked_state.array.len() > 0 {
            // If the array contains something, just return the next
            // items in the vector in a cyclic fashion.
            if locked_state.index >= locked_state.array.len() {
                locked_state.index = 0;
            }
            let result = locked_state.array[locked_state.index];
            locked_state.index += 1;
            Poll::Ready(Some(Ok(result)))
        } else {
            // If the array is empty, just ask to be notified in the
            // future and say that the stream is not ready.
            //
            // If we do not ask to be notified in the future
            // explicitly, this future will never be scheduled again.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Debug)]
struct State {
    array: Vec<i32>,
    index: usize,
}

impl State {
    fn new() -> Self {
        Self {
            array: Vec::new(),
            index: 0,
        }
    }
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(State::new()));

    // This future just produces one number each second from the
    // stream.
    let numbers_fut = {
        let mut numbers =
            MyStream::new(shared_state.clone()).zip(interval(Duration::from_millis(1000)));
        async move {
            while let Some((number, _instant)) = numbers.next().await {
                println!("got number {:?}", number);
            }
        }
    };

    // This future run every 5 seconds and insert an item into the
    // array, up to a limit of 5, and then clears the array again.
    let on_off_fut = {
        let state = shared_state.clone();
        // This variable will retain the state between invocations of
        // the closure below.
        let mut val = 0;
        async move {
            let mut ticks = interval(Duration::from_millis(5000));
            while let Some(_instant) = ticks.next().await {
                let mut locked_state = state.lock().unwrap();
                if locked_state.array.len() < 5 {
                    println!("pushing {} on array", val);
                    locked_state.array.push(val);
                    val += 1;
                } else {
                    println!("clearing array");
                    locked_state.array.clear();
                }
            }
        }
    };

    let _ = future::join(tokio::spawn(numbers_fut), tokio::spawn(on_off_fut)).await;
}
