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

// An example of how to chain several closures to process the same set
// of items in a stream. One of the futures in the middle will return
// an error when the value is too large, to check that we can abort
// the processing in the expected way.

extern crate futures;
extern crate tokio;

use futures::{stream, Stream};
use tokio::prelude::*;

/// Function to create a Fibonacci stream.
fn fibonacci() -> impl Stream<Item = u64, Error = ()> {
    stream::unfold((1, 1), |(curr, next)| {
        let new_next = curr + next;
        Some(Ok((curr, (next, new_next))))
    })
}

fn main() {
    tokio::run({
        fibonacci()
            .take(10)
            .and_then(|n| {
                print!("Square: {} => ", n);
                if n < 10 {
                    Ok(n * n)
                } else {
                    Err(())
                }
            })
            .for_each(|n| {
                println!("{}", n);
                Ok(())
            })
            .map_err(|e| println!("error: {:?}", e))
            .map(|_| ())
    });
}
