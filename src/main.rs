use std::thread::spawn;
use std::io;
use rand::distr::{Distribution,StandardUniform};
fn main() {
    println!("enter number of whole numbers in array");
    let mut input=String::new();
    io::stdin().read_line(&mut input).expect("read line fail");
    let n:usize=input.trim().parse().expect("not number");
    let nums:Vec<u64>=StandardUniform.sample_iter(rand::rng()).take(n).collect();
    
}
