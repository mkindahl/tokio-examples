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

// Example that demonstrates how to create a cycle stream and zip that
// with an Interval stream.

use futures::stream::Stream;
use futures::{Async, Poll};
use std::iter::Cycle;
use std::time::{Duration, Instant};
use tokio::timer::Interval;

struct IterCycle<I> {
    iter: Cycle<I>,
}

fn iter_cycle<I>(i: I) -> IterCycle<I::IntoIter>
where
    I: IntoIterator,
    I::IntoIter: Clone,
{
    IterCycle {
        iter: i.into_iter().cycle(),
    }
}

impl<I> Stream for IterCycle<I>
where
    I: Iterator + Clone,
{
    type Item = <I as Iterator>::Item;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(self.iter.next()))
    }
}

fn main() {
    // iter_cycle return a stream with Error = (), which means that we
    // need to map the error from the Interval stream to () as well.
    let primes = iter_cycle(vec![2, 3, 5, 7, 11, 13])
        .take(20)
        .zip(
            Interval::new(Instant::now(), Duration::from_millis(500))
                .map_err(|err| println!("Error: {}", err)),
        )
        .for_each(|(number, instant)| {
            println!("fire; number={}, instant={:?}", number, instant);
            Ok(())
        });
    tokio::run(primes);
}
