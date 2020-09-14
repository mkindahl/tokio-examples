use futures::{stream, Stream};

/// Generate a stream of Fibonacci numbers
///
///
pub fn fibonacci() -> impl Stream<Item = u64> {
    stream::unfold((1, 1), |(curr, next)| async move {
        Some((curr, (next, curr + next)))
    })
}
