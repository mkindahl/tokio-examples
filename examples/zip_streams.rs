extern crate futures;

use futures::{stream, Stream};
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer;

/// Produce a stream of Fibonacci numbers.
///
/// Note that the error for the stream is the same as what the
/// interval timer for Tokio produces.
fn fibonacci() -> impl Stream<Item = u64, Error = Error> {
    stream::unfold((1, 1), |(curr, next)| {
        let new_next = curr + next;
        if new_next < 100 {
            Some(Ok((curr, (next, new_next))))
        } else {
            Some(Err(Error::TooHigh))
        }
    })
}

#[derive(Debug)]
enum Error {
    TooHigh,
    Timer(timer::Error),
}

fn main() {
    let task = timer::Interval::new(Instant::now(), Duration::from_millis(500))
        .map_err(|err| Error::Timer(err))
        .zip(fibonacci())
        .for_each(|(instant, number)| {
            println!("fire; instant={:?}, number={}", instant, number);
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    tokio::run(task);
}
