use std::thread;
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::distr::{Distribution,StandardUniform};
use std::time::{Duration,Instant};

const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}s", secs, millis)
}

fn print_progress(label: &str, done: u64, total: u64, start: Instant) {
    let width=28usize;
    let percent=if total==0 { 1.0 } else { done as f64 / total as f64 };
    let filled=(percent * width as f64).round() as usize;
    let elapsed=start.elapsed();
    let eta=if done==0 {
        Duration::from_secs(0)
    } else {
        let per_item=elapsed.as_secs_f64() / done as f64;
        Duration::from_secs_f64(per_item * (total - done) as f64)
    };
    print!("\r{}{}{} [{}{}] {}/{} {}elapsed:{}{} ETA:{} {}{}", CYAN, label, RESET, GREEN, "#".repeat(filled), done, total, YELLOW, format_duration(elapsed), RESET, YELLOW, format_duration(eta), RESET);
    io::stdout().flush().expect("flush fail");
    if done>=total {
        println!();
    }
}

fn watch_progress(label: &'static str, progress: Arc<AtomicU64>, total: u64, start: Instant) -> thread::JoinHandle<()> {
    thread::spawn(move ||{
        loop{
            let done=progress.load(Ordering::Relaxed);
            print_progress(label, done, total, start);
            if done>=total{
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    })
}

pub fn add() {
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
    let progress=Arc::new(AtomicU64::new(0));
    let watcher=watch_progress("addition multi-threaded", Arc::clone(&progress), n as u64, start_multi);
    thread::scope(|s| {
        let progress_t1 = Arc::clone(&progress);
        let h1 = s.spawn(move || {
            let mut sum = 0u64;
            for &x in n1 {
                sum = sum.wrapping_add(x);
                progress_t1.fetch_add(1, Ordering::Relaxed);
            }
            sum
        });
        let progress_t2 = Arc::clone(&progress);
        let h2 = s.spawn(move || {
            let mut sum = 0u64;
            for &x in n2 {
                sum = sum.wrapping_add(x);
                progress_t2.fetch_add(1, Ordering::Relaxed);
            }
            sum
        });
        let progress_t3 = Arc::clone(&progress);
        let h3 = s.spawn(move || {
            let mut sum = 0u64;
            for &x in n3 {
                sum = sum.wrapping_add(x);
                progress_t3.fetch_add(1, Ordering::Relaxed);
            }
            sum
        });
        let progress_t4 = Arc::clone(&progress);
        let h4 = s.spawn(move || {
            let mut sum = 0u64;
            for &x in n4 {
                sum = sum.wrapping_add(x);
                progress_t4.fetch_add(1, Ordering::Relaxed);
            }
            sum
        });
        s1=h1.join().expect("Thread panicked");
        s2=h2.join().expect("Thread panicked");
        s3=h3.join().expect("Thread panicked");
        s4=h4.join().expect("Thread panicked");
    });
    watcher.join().expect("Thread panicked");
    let sum: u64 = s1
    .wrapping_add(s2)
    .wrapping_add(s3)
    .wrapping_add(s4);
    println!("{}time taken for multi threaded:{} {}", YELLOW, RESET, format_duration(start_multi.elapsed()));
    let start_single=Instant::now();
    let single_progress=Arc::new(AtomicU64::new(0));
    let single_watcher=watch_progress("addition single-threaded", Arc::clone(&single_progress), n as u64, start_single);
    let mut single_sum=0u64;
    for &x in &nums {
        single_sum=single_sum.wrapping_add(x);
        single_progress.fetch_add(1, Ordering::Relaxed);
    }
    single_watcher.join().expect("Thread panicked");
    println!("{}time taken for single threaded:{} {}", YELLOW, RESET, format_duration(start_single.elapsed()));
    println!("the multi-threaded sum is:{}",sum);
    println!("the single-threaded sum is:{}",single_sum);
}