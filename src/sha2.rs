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

// Precomputed combinations of 3 positions out of 8
const POS_COMBINATIONS: [[usize; 3]; 56] = [
    [0,1,2], [0,1,3], [0,1,4], [0,1,5], [0,1,6], [0,1,7],
    [0,2,3], [0,2,4], [0,2,5], [0,2,6], [0,2,7], [0,3,4],
    [0,3,5], [0,3,6], [0,3,7], [0,4,5], [0,4,6], [0,4,7],
    [0,5,6], [0,5,7], [0,6,7], [1,2,3], [1,2,4], [1,2,5],
    [1,2,6], [1,2,7], [1,3,4], [1,3,5], [1,3,6], [1,3,7],
    [1,4,5], [1,4,6], [1,4,7], [1,5,6], [1,5,7], [1,6,7],
    [2,3,4], [2,3,5], [2,3,6], [2,3,7], [2,4,5], [2,4,6],
    [2,4,7], [2,5,6], [2,5,7], [2,6,7], [3,4,5], [3,4,6],
    [3,4,7], [3,5,6], [3,5,7], [3,6,7], [4,5,6], [4,5,7],
    [4,6,7], [5,6,7]
];

fn i2s(mut index: u64, buf: &mut [u8; 8]) {
    // 1. Extract 5 lowercase values (26^5)
    let mut lower_vals = [0u8; 5];
    for i in (0..5).rev() {
        lower_vals[i] = (index % 26) as u8;
        index /= 26;
    }

    // 2. Extract 3 uppercase values (26^3)
    let mut upper_vals = [0u8; 3];
    for i in (0..3).rev() {
        upper_vals[i] = (index % 26) as u8;
        index /= 26;
    }

    // 3. Select which positions are uppercase (56 choices)
    let combo_idx = (index % 56) as usize;
    let upper_positions = POS_COMBINATIONS[combo_idx];

    // 4. Populate buffer
    let mut u_ptr = 0;
    let mut l_ptr = 0;
    for i in 0..8 {
        if upper_positions.contains(&i) {
            buf[i] = b'A' + upper_vals[u_ptr];
            u_ptr += 1;
        } else {
            buf[i] = b'a' + lower_vals[l_ptr];
            l_ptr += 1;
        }
    }
}
fn hex_decode_32(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i*2..i*2+2], 16).unwrap();
    }
    out
}

pub fn sha2() {
    let num_threads: usize = thread::available_parallelism().unwrap().get();
    println!("starting sha256 brute force with {} threads", num_threads);
    let total_combs = 56*26u64.pow(8);
    let target_hex = "4a8d2dcdcc90b1ee98bc16f7097163854f879654e44ec4ffc71e54afcf5d0c13";
    let target_bytes = hex_decode_32(target_hex);
    let progress = Arc::new(AtomicU64::new(0));
    let found = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    let chunk = (total_combs + num_threads as u64 - 1) / num_threads as u64;
    let watcher = watch_progress("sha2 brute force", Arc::clone(&progress), total_combs, start);
    thread::scope(|s| {
        for t in 0..num_threads {
            let begin = t as u64 * chunk;
            let end = ((t as u64 + 1) * chunk).min(total_combs);
            let target_bytes = &target_bytes;
            let progress = Arc::clone(&progress);
            let found = Arc::clone(&found);
            s.spawn(move || {
                let mut buf = [0u8; 8];
                let mut hash_buf = [0u8; 32];
                let mut local = 0u64;
                for index in begin..end {
                    if found.load(Ordering::Relaxed) == 1 {
                        break;
                    }
                    i2s(index, &mut buf);
                    unsafe {
                        openssl_sys::SHA256(buf.as_ptr(), 8, hash_buf.as_mut_ptr());
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
    println!("{}time taken for sha2 brute force:{} {}", YELLOW, RESET, format_duration(start.elapsed()));
}