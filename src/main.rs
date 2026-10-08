use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::io;
use openssl_sys;
use rand::distr::{Distribution,StandardUniform};
use std::time::Instant;
use hex;
fn add() {
    println!("enter number of whole numbers in array");
    let mut input=String::new();
    io::stdin().read_line(&mut input).expect("read line fail");
    let n:usize=input.trim().parse().expect("not number");
    let nums:Vec<u64>=StandardUniform.sample_iter(rand::rng()).take(n).collect();
    let n1:&[u64]=&nums[0..n/4];
    let n2:&[u64]=&nums[n/4..n/2];
    let n3:&[u64]=&nums[n/2..3*n/4];
    let n4:&[u64]=&nums[3*n/4..];
    let mut s1=0u64;
    let mut s2=0u64;
    let mut s3=0u64;
    let mut s4=0u64;
    let start_multi=Instant::now();
    thread::scope(|s| {
        s.spawn(||{
            s1=n1.iter().fold(0u64, |acc, &x| acc.wrapping_add(x));
        });
        s.spawn(||{
            s2=n2.iter().fold(0u64, |acc, &x| acc.wrapping_add(x));
        });
        s.spawn(||{
            s3=n3.iter().fold(0u64, |acc, &x| acc.wrapping_add(x));
        });
        s.spawn(||{
            s4=n4.iter().fold(0u64, |acc, &x| acc.wrapping_add(x));
        });

    });
    let sum: u64 = s1
    .wrapping_add(s2)
    .wrapping_add(s3)
    .wrapping_add(s4);
    println!("time taken for multi threaded: {:?}", start_multi.elapsed());
    let start_single=Instant::now();
    let single_sum: u64 = nums.iter().fold(0u64, |acc, &x| acc.wrapping_add(x));
    println!("time taken for single threaded: {:?}", start_single.elapsed());
    println!("the multi-threaded sum is:{}",sum);
    println!("the single-threaded sum is:{}",single_sum);
}
fn i2s(mut index:u64)-> String {
    let mut buf=[0u8;7];
    for i in (0..7).rev(){
        buf[i]=b'a'+(index%26) as u8;
        index/=26;
    }
    unsafe{String::from_utf8_unchecked(buf.to_vec())}

}
fn sha(){
    let counter=Arc::new(AtomicU64::new(0));
    let num_threads: usize=thread::available_parallelism().unwrap().get();
    println!("starting sha256 brute force with {} threads",num_threads);
    let total_combs=26u64.pow(7);
    let hash: Arc<String>=Arc::new("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string());
    let mut handles=vec![];
    for _ in 0..num_threads {
        let counter=Arc::clone(&counter);
        let hash=Arc::clone(&hash);
        let handle=thread::spawn(move ||{
            loop{
                let index=counter.fetch_add(1,Ordering::Relaxed);
                if index>=total_combs{
                    break;
                }
                let s: String=i2s(index);
                let c_str: std::ffi::CString=std::ffi::CString::new(s.as_str()).unwrap();
                unsafe{
                    let mut hash_buf=[0u8;32];
                    openssl_sys::SHA256(c_str.as_ptr() as *const u8,7,hash_buf.as_mut_ptr());
                    let genhash = hex::encode(hash_buf);
                    if genhash==*hash{
                        println!("found matching string: {}",s);
                        break;
                    }
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
fn main() {
    add();
    sha();
}