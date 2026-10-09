use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::io::Write;
use std::time::{Duration,Instant};
use openssl_sys;
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}s", secs, millis)
}

fn format_rate(rate: f64) -> String {
    if rate >= 1_000_000_000.0 {
        format!("{:.2}B/s", rate / 1_000_000_000.0)
    } else if rate >= 1_000_000.0 {
        format!("{:.2}M/s", rate / 1_000_000.0)
    } else if rate >= 1_000.0 {
        format!("{:.2}K/s", rate / 1_000.0)
    } else {
        format!("{:.2}/s", rate)
    }
}

fn print_progress(label: &str, done: u64, total: u64, start: Instant) {
    let width=28usize;
    let percent=if total==0 { 1.0 } else { done as f64 / total as f64 };
    let filled=(percent * width as f64).round() as usize;
    let elapsed=start.elapsed();
    let rate=if elapsed.as_secs_f64()==0.0 { 0.0 } else { done as f64 / elapsed.as_secs_f64() };
    let eta=if done==0 {
        Duration::from_secs(0)
    } else {
        let per_item=elapsed.as_secs_f64() / done as f64;
        Duration::from_secs_f64(per_item * (total - done) as f64)
    };
    print!("\r{}{}{} [{}{}{}] {}{:>5.1}%{} {}/{} {}elapsed:{}{} {}rate:{}{} ETA:{} {}{}",
        CYAN, label, RESET,
        GREEN, "#".repeat(filled), " ".repeat(width-filled),
        MAGENTA, percent*100.0, RESET,
        done, total,
        YELLOW, RESET, format_duration(elapsed),
        YELLOW, RESET, format_rate(rate),
        YELLOW, format_duration(eta), RESET);
    std::io::stdout().flush().expect("flush fail");
    if done>=total {
        println!();
    }
}

fn watch_progress(label: &'static str, progress: Arc<AtomicU64>, total: u64, start: Instant) -> thread::JoinHandle<()> {
    thread::spawn(move ||{
        loop{
            let done=progress.load(Ordering::Relaxed);
            print_progress(label, done.min(total), total, start);
            if done>=total{
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    })
}

fn i2s(mut index: u64, buf: &mut [u8; 7]) {
    for i in (0..7).rev() {
        buf[i] = b'a' + (index % 26) as u8;
        index /= 26;
    }
}
fn hex_decode_32(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i*2..i*2+2], 16).unwrap();
    }
    out
}

pub fn sha1() {
    let num_threads: usize = thread::available_parallelism().unwrap().get();
    println!("starting sha256 brute force with {} threads", num_threads);
    let total_combs = 26u64.pow(7);
    let target_hex = "32adca3bde67a61815896740ee954b0b3dea15f6851f1b35079698729081e2ba";
    let target_bytes = hex_decode_32(target_hex);

    let progress = Arc::new(AtomicU64::new(0));
    let found = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    let chunk = (total_combs + num_threads as u64 - 1) / num_threads as u64;
    let watcher = watch_progress("sha1 brute force", Arc::clone(&progress), total_combs, start);

    thread::scope(|s| {
        for t in 0..num_threads {
            let begin = t as u64 * chunk;
            let end = ((t as u64 + 1) * chunk).min(total_combs);
            let target_bytes = &target_bytes;
            let progress = Arc::clone(&progress);
            let found = Arc::clone(&found);
            s.spawn(move || {
                let mut buf = [0u8; 7];
                let mut hash_buf = [0u8; 32];
                let mut local = 0u64;
                for index in begin..end {
                    if found.load(Ordering::Relaxed) == 1 {
                        break;
                    }
                    i2s(index, &mut buf);
                    unsafe {
                        openssl_sys::SHA256(buf.as_ptr(), 7, hash_buf.as_mut_ptr());
                    }
                    local += 1;
                    if local >= 100_000 {
                        progress.fetch_add(local, Ordering::Relaxed);
                        local = 0;
                    }
                    if hash_buf == *target_bytes {
                        found.store(1, Ordering::Relaxed);
                        progress.fetch_add(local, Ordering::Relaxed);
                        progress.store(total_combs, Ordering::Relaxed);
                        println!(
                            "\n{}found matching string:{} {}",
                            GREEN, RESET, std::str::from_utf8(&buf).unwrap()
                        );
                        return;
                    }
                }
                progress.fetch_add(local, Ordering::Relaxed);
            });
        }
    });

    if found.load(Ordering::Relaxed) == 0 {
        progress.store(total_combs, Ordering::Relaxed);
    }
    watcher.join().expect("Thread panicked");
    println!("{}time taken for sha1 brute force:{} {}", YELLOW, RESET, format_duration(start.elapsed()));
}