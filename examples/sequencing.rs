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
