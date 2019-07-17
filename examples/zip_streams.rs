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

// Example to demonstrate how to zip two streams that are both
// infinite and have different types for `Error`. The fibonacci stream
// has `()` for `Error` while the interval stream has `tokio::timer::Error`.
//
// We handle this by mapping the error type for each stream to a
// common error type, which also means that we need to map the "no
// error" type to some sort of error enumeration, but this error can
// never occur.

extern crate futures;

use futures::{stream, Stream};
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer;

// Produce a stream of Fibonacci numbers.
//
// Note that the error for the stream is the same as what the
// interval timer for Tokio produces.
fn fibonacci() -> impl Stream<Item = u64, Error = ()> {
    stream::unfold((1, 1), |(curr, next)| {
        let new_next = curr + next;
        Some(Ok((curr, (next, new_next))))
    })
}

#[derive(Debug)]
enum Error {
    Unused,
    Timer(timer::Error),
}

fn main() {
    let task = timer::Interval::new(Instant::now(), Duration::from_millis(500))
        .map_err(|err| Error::Timer(err))
        .zip(fibonacci().map_err(|()| Error::Unused))
        .for_each(|(instant, number)| {
            println!("fire; instant={:?}, number={}", instant, number);
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    tokio::run(task);
}
