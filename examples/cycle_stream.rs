use core::marker;
use futures::stream::Stream;
use futures::{Async, Poll};
use std::iter::Cycle;
//use std::time::{Duration, Instant};
//use tokio::timer::Interval;

pub struct IterCycle<I, E> {
    iter: Cycle<I>,
    _marker: marker::PhantomData<fn() -> E>,
}

pub fn iter_cycle<I, E>(i: I) -> IterCycle<I::IntoIter, E>
where
    I: IntoIterator,
    I::IntoIter: Clone,
{
    IterCycle {
        iter: i.into_iter().cycle(),
        _marker: marker::PhantomData,
    }
}

impl<I, E> Stream for IterCycle<I, E>
where
    I: Iterator + Clone,
{
    type Item = <I as Iterator>::Item;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<I::Item>, E> {
        Ok(Async::Ready(self.iter.next()))
    }
}

fn main() {
    let primes = iter_cycle(vec![2, 3, 5, 7, 11, 13])
        .take(20)
        .for_each(|number| {
            println!("fire; number={}", number);
            Ok(())
        });
    tokio::run(primes);
}
