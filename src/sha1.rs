use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::io::Write;
use hex;
use std::time::{Duration,Instant};
use openssl_sys;
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
    std::io::stdout().flush().expect("flush fail");
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

fn i2s(mut index:u64)-> String {
    let mut buf=[0u8;7];
    for i in (0..7).rev(){
        buf[i]=b'a'+(index%26) as u8;
        index/=26;
    }
    unsafe{String::from_utf8_unchecked(buf.to_vec())}

}

pub fn sha1(){
    let counter=Arc::new(AtomicU64::new(0));
    let num_threads: usize=thread::available_parallelism().unwrap().get();
    println!("starting sha256 brute force with {} threads",num_threads);
    let total_combs=26u64.pow(7);
    let hash: Arc<String>=Arc::new("32adca3bde67a61815896740ee954b0b3dea15f6851f1b35079698729081e2ba".to_string());
    let start=Instant::now();
    let watcher=watch_progress("sha1 brute force", Arc::clone(&counter), total_combs, start);
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
    watcher.join().expect("Thread panicked");
    println!("{}time taken for sha1 brute force:{} {}", YELLOW, RESET, format_duration(start.elapsed()));
}