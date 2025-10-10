fn main() {
    let mut x = 0;
    while x < 1000000000 {
        x+=1;
        if x%100000000 == 0 {
            println!("{}", x);
        }
    }
}
