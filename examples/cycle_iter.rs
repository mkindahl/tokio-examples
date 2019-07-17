fn main() {
    let primes = vec![2, 3, 5, 7, 11, 13];
    for (index, prime) in (1..10).zip(primes.iter().cycle()) {
        println!("{}: {}", index, prime);
    }
}
