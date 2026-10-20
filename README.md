# Parallel Computing Challenge; Web Club NITK Recruitments 2026

Submission by **Dikshit Rishi Jain** (First Year, NITK Surathkal)

Mentors: Nimesh Acharya ([@Radonoxius](https://github.com/Radonoxius)), Ranjit Tanneru ([@AmissDrake](https://github.com/AmissDrake))

Repo: https://github.com/DikshitRJ/webclub-system-parallel-2026

---

## Table of Contents

- [TL;DR](#tldr)
- [Repo Structure](#repo-structure)
- [Building & Running](#building--running)
- [Challenge I; Multithreaded Array Summation](#challenge-i--multithreaded-array-summation)
  - [How the Work Is Split](#how-the-work-is-split)
  - [Why `Arc<AtomicU64>` and Not Just a Mutex](#why-arcatomicu64-and-not-just-a-mutex)
  - [Benchmark Numbers (Warts and All)](#benchmark-numbers-warts-and-all)
- [Challenge II; SHA-256 Brute Force](#challenge-ii--sha-256-brute-force)
  - [Turning a Number Into a Candidate String](#turning-a-number-into-a-candidate-string)
  - [The Overhead Lesson](#the-overhead-lesson)
  - [Fighting the Borrow Checker](#fighting-the-borrow-checker)
  - [Taking It to the NITK Lab Server](#taking-it-to-the-nitk-lab-server)
  - [Results](#results)
  - [Open Question: Would C Be Faster?](#open-question-would-c-be-faster)
- [Bonus; Android/Termux Cross-Compile](#bonus--androidtermux-cross-compile)
- [What I'd Do Differently](#what-id-do-differently)
- [A Note on AI Use](#a-note-on-ai-use)
- [Acknowledgements](#acknowledgements)

---

## TL;DR

Both challenges are in one Rust binary with a little arrow-key menu (`add` / `sha1` / `sha2`) built with `crossterm`. Challenge I sums a huge array of `u64`s with 4 threads and compares it against doing it single-threaded. Challenge II brute-forces two SHA-256 hashes using `openssl-sys` across every CPU thread available.

Cracked hash 1 (`threadz`). Hash 2 is still out there; see [why](#results). Didn't get to the Android bonus.

I wrote the addition and SHA-1 logic myself first as plain, unglamorous scripts that just worked. Then I had AI refactor that into cleaner code and bolt on the terminal UI (progress bars, ETA, the works), because watching a raw `println!` scroll for four hours is not my idea of a good time. More on that split further down.

## Repo Structure

Everything lives in one Cargo project; I didn't split Challenge I and II into separate crates/folders, they're separate modules instead:

```
.
├── .vscode/
│   └── launch.json
├── src/
│   ├── main.rs        # arrow-key menu, picks which challenge to run
│   ├── addition.rs     # Challenge I
│   ├── sha1.rs          # Challenge II, hash 1 (7 lowercase chars)
│   └── sha2.rs          # Challenge II, hash 2 (8 chars, mixed case)
├── Cargo.toml
├── Cargo.lock
└── .gitignore
```

## Building & Running

```bash
cargo build --release
./target/release/parallel-computing
```

Use the up/down arrows to pick a challenge, hit enter. For `add`, it'll ask you how many numbers you want in the array (go big, like 8-9 digits, or it finishes before you can blink).

For the hash-cracking bits specifically, I cross-compiled to a static musl binary so I could run the exact same artifact on my own machine and on a completely different server without worrying about glibc versions:

```bash
cross build --release --target x86_64-unknown-linux-musl
```

## Challenge I; Multithreaded Array Summation

### How the Work Is Split

`addition.rs` generates `n` random `u64`s, then slices the array into four quarters exactly like the brief asks; thread `i` gets the range `i*N/4` to `(i+1)*N/4`:

```rust
let n1: &[u64] = &nums[0..n/4];
let n2: &[u64] = &nums[n/4..n/2];
let n3: &[u64] = &nums[n/2..3*n/4];
let n4: &[u64] = &nums[3*n/4..];
```

Each of the four threads (spawned inside a `thread::scope`) just walks its own slice and accumulates a local `sum` with `wrapping_add`, no shared state involved in the actual arithmetic. Once all four join, I add the four partial sums together and compare that against a plain sequential loop over the whole array, timing both.

### Why `Arc<AtomicU64>` and Not Just a Mutex

The actual sum doesn't need any synchronization; each thread owns its slice outright, there's nothing to race over. The thing that *does* need to be shared is the progress counter driving the little terminal progress bar, because a separate watcher thread polls it every 100ms to redraw the UI while the worker threads are still running.

That watcher thread is spawned with a plain `thread::spawn`, not inside the scope, which means Rust requires whatever it captures to be `'static`; it has to own its data outright, it can't just borrow a reference off the stack like the scoped worker threads can. So the counter needs to be something with shared ownership that multiple independent threads can each hold a handle to and mutate safely without a race. That's exactly what `Arc<AtomicU64>` gives you: `Arc` handles the shared-ownership part (clone it, hand a copy to every thread, the underlying data lives as long as any clone does), and `AtomicU64` handles the "many threads increment this at once" part without needing a lock.

Could've used a `Mutex<u64>` wrapped in an `Arc` instead, but that means every single increment has to acquire and release a lock; way more overhead than a lock-free atomic add for something this hot. `Arc<AtomicU64>` is really just "shared ownership of a value that's safe to mutate concurrently," which is precisely the shape of the problem here.

### Benchmark Numbers (Warts and All)

Here's an actual run on my own machine:

```
enter number of whole numbers in array
10000000
addition multi-threaded [####...] 10000000/10000000 elapsed:0.100s ETA: 0.000s
time taken for multi threaded: 0.100s
addition single-threaded [####...] 10000000/10000000 elapsed:0.100s ETA: 0.000s
time taken for single threaded: 0.100s
```

At 10 million elements, multi and single threaded both land at 0.1s; basically tied, probably just timer resolution at that point.

```
enter number of whole numbers in array
100000000
addition multi-threaded [####...] 100000000/100000000 elapsed:1.004s ETA: 0.000s
time taken for multi threaded: 1.004s
addition single-threaded [####...] 100000000/100000000 elapsed:0.801s ETA: 0.000s
time taken for single threaded: 0.801s
```

At 100 million, the multithreaded version was actually *slower*; 1.004s vs 0.801s. Not the flex I was going for. I was running this on my daily driver, a 2-core/4-thread laptop, with a dozen other windows open fighting for the same cores. Spinning up 4 threads plus a watcher thread on a machine that barely has 4 threads to begin with, while the OS is also trying to schedule everything else I had open, means the threads spend real time just waiting for a free core instead of actually summing anything. The work per thread here is cheap enough that thread spawn/scheduling overhead eats whatever parallelism should've bought me. On a machine with real headroom (or the 80-thread server I used for Challenge II) I'd expect this to flip hard in multithreading's favor.

I also tried 1,000,000,000 elements and ended up Ctrl-C-ing out of it; my laptop crashed, as the randomly generate u64 sample space itself would be 8gb, and thats just how much RAM I have.

## Challenge II; SHA-256 Brute Force

### Turning a Number Into a Candidate String

Rather than literally keeping a string and incrementing it like an odometer, both `sha1.rs` and `sha2.rs` treat the whole search space as one giant number and decode each index into a candidate string on the fly.

For hash 1 (7 lowercase letters), `i2s` just does base-26 conversion; repeatedly mod/div by 26 to peel off one letter at a time, same idea as converting a number to any other base, just with `a`-`z` as the digits.

For hash 2 (8 chars, exactly 3 uppercase and 5 lowercase) it's a bit more involved, because the positions of the uppercase letters also vary. I precomputed every way to choose 3 positions out of 8 (there are exactly 56; that's `C(8,3)`) into a lookup table, `POS_COMBINATIONS`. Then a single index gets split three ways: one chunk picks the 5 lowercase letters (base-26), one chunk picks the 3 uppercase letters (base-26), and the last chunk (mod 56) picks which positions are uppercase. Multiply those three search dimensions together and you get the total space: `56 * 26^8`.

That number matters later, so hold onto it.

### The Overhead Lesson

My first version of the brute-forcer shared one `Arc<AtomicU64>` as a "next index to try" counter; every thread, for every single candidate, would atomically fetch-and-increment that counter to grab its next string to test. Sounds reasonable on paper. In practice it was awful, because hashing a 7-character string is *stupidly* fast; we're talking nanoseconds; so the atomic fetch-add itself (which has to bounce across CPU cores to stay coherent) ended up being a bigger cost than the actual hashing. Finding hashes was never the slow part; hammering a shared memory location from 80 threads at once was.

The fix was to stop sharing anything for work assignment entirely. Each thread gets handed one fixed, contiguous range of the search space up front (`begin..end`, computed once from `total_combs / num_threads`) and just chews through it with zero coordination needed. The only atomic traffic left is the progress counter for the UI, and even that only gets touched once every 100,000 hashes instead of every single one:

```rust
local += 1;
if local >= 100_000 {
    progress.fetch_add(local, Ordering::Relaxed);
    local = 0;
}
```

Batching that update was enough to make the atomic overhead basically disappear.

### Fighting the Borrow Checker

Moving things into threads was its own adventure. The classic gotcha: once a variable gets moved into one thread's `move ||` closure, you can't touch it anywhere else afterwards; ownership just isn't there anymore, and `cargo check` will absolutely let you know about it. Took a few rounds of "why does this work in my head but not in the compiler" before it clicked that I needed a fresh reference per thread (`let target_bytes = &target_bytes;` right before the closure, so each thread captures its own borrow instead of fighting over the original) rather than trying to share the one variable across all four spawns.

Using `thread::scope` for the worker threads helped a lot here too; scoped threads are guaranteed to finish before the scope ends, so the borrow checker will happily let you hand out plain references to stack data without needing `Arc` or `clone()` everywhere, which is a lot less ceremony than non-scoped threads demand.

### Taking It to the NITK Lab Server

My own laptop was never going to crack a billion-plus SHA-256 hashes in a reasonable amount of time. NITK's firstyear-lab server, on the other hand, has a 40-core/80-thread Intel Xeon sitting there; nominally provisioned for over a thousand first-years to timeshare, not for one guy to pin at 100% for hours brute-forcing a recruitment challenge. Which is, to be clear, probably not what that machine is *for*. But it was sitting right there, so.

Getting the binary onto it was its own small saga. My local machine runs openSUSE Tumbleweed; the lab server runs Ubuntu 18, with a glibc old enough that a binary built fresh on my machine just wasn't going to run there. And since the program links against OpenSSL's C library for the actual hashing (`openssl` + `openssl-sys` with the `vendored` feature, which compiles OpenSSL from source at build time rather than linking whatever's already on the system), it needs a real C compiler in the build pipeline too; this isn't a pure-Rust static binary situation.

`cross` solved both problems at once: `cross build --release --target x86_64-unknown-linux-musl` builds inside a Docker container with the right musl C cross-toolchain already set up, and spits out a fully static binary. Static means no glibc version to worry about at all; the same binary ran fine on my machine and on the Ubuntu 18 server without any fuss. `scp`'d it over, ran it, and watched `htop` light up all 80 threads at 100%.

This all happened during an all-nighter I'd originally planned for studying for my Intro to Engineering midsem. Studying did not happen. What did happen was me genuinely jumping up and down when the thing actually started running on the server; I did not expect that to just *work* on the first real attempt, let alone chew through billions of hashes a second on hardware that size.

### Results

**Hash 1 cracked: `threadz`.**

The progress bar's ETA said around 4 hours; it actually found the match in about 1.5 hours (random target, so landing before the full ETA checks out). Slower than I expected going in, though; 80 threads sounds like it should demolish 8 billion combinations almost instantly, but that particular Xeon doesn't have the SHA-NI instruction extensions that let modern CPUs do SHA-256 compression rounds in hardware. Without that, OpenSSL falls back to computing everything in software, so even with 80 threads going flat out, each one is doing meaningfully more work per hash than it would on newer hardware.

**Hash 2: not cracked.** And honestly, the math says it was never going to happen in a reasonable window. Hash 1's search space is `26^7 ≈ 8.03 billion`. Hash 2's is `56 * 26^8 ≈ 11.69 trillion`; exactly **1,456 times larger**. If hash 1 took ~1.5 hours at this throughput, hash 2 scales out to roughly `1.5 × 1,456 ≈ 2,184 hours`, which is about **91 days**. That lines up almost exactly with the "ETA: days" the program was reporting. I let it run for about 5 minutes, saw that number, and gave up. Tried to find something to optimize and came up empty; I don't know C well enough to try rewriting the hot loop there, so I ran out of ideas on my end.

### Open Question: Would C Be Faster?

Genuinely curious what you two think here. My gut says probably not by much, for one specific reason: `openssl-sys` is just FFI bindings straight into OpenSSL's actual C implementation; when my Rust code calls `openssl_sys::SHA256`, it's running the *exact same* compiled C code a hand-written C version would call too. The hot loop itself doesn't allocate anything per iteration (the buffers are stack-allocated once and reused), so there isn't obvious Rust-side overhead sitting between the loop and the hash call for C to strip away.

If that's right, the real bottleneck is the CPU's instruction set (no SHA-NI, so everything's done the slow way) rather than anything language-level; which would mean a C rewrite gets you the same wall-clock time, not a faster one. The actual lever would probably be something like GPU-based hashing (SHA-256 parallelizes extremely well on GPUs) or just a newer CPU. But I'd love to be wrong here if there's a C-specific trick I'm missing; SIMD intrinsics, a leaner OpenSSL build flag, anything along those lines.

## Bonus; Android/Termux Cross-Compile

Didn't get to this one. Between the two main challenges and the all-nighter it took to get Challenge II running properly, I ran out of time before I could set up the Android NDK toolchain and test it under Termux. Left for a future attempt.

## What I'd Do Differently

- Start the search-space math earlier; I could've sanity-checked hash 2's feasibility with a calculator before burning server time on it, instead of finding out live.
- Actually learn enough C to test the "would C help" theory myself instead of just guessing.
- Pick a less academically important night for the "quick side project that spirals" experience.

Genuinely had a lot of fun with this, overhead regrets and all.

## A Note on AI Use

I wrote the core logic for Challenge I and the hash 1 brute-forcer myself first, as plain working scripts with no UI. It must be at one of the previous comits. From there, I had AI refactor that into the current structure and add the `crossterm` terminal UI (progress bars, ETA, the arrow-key menu); I wanted something nicer to look at during a multi-hour run than a wall of scrolling prints, and figured that part didn't need to be hand-rolled from scratch. This README itself was also written with AI assistance, based on my own notes and actual terminal output from the runs described above.

## Acknowledgements

Thanks to Nimesh Acharya and Ranjit Tanneru for mentoring this challenge, and to Web Club NITK for putting it together; this was a genuinely fun excuse to actually understand what `Arc` and atomics are for instead of just copy-pasting them.
