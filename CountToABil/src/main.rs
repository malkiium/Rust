use std::time::Instant;

fn main() {
    let start = Instant::now();
    let mut x = 0;
    while x < 1000000000 {
        x += 1;
    }
    let elapsed = start.elapsed();
    println!("Final value: {}, Time elapsed: {:?}", x, elapsed);
}