# Parallel Computing Challenge

**Tags**:
[C](https://en.wikipedia.org/wiki/C_(programming_language)),
[Rust](https://rust-lang.org/),
[Multi-processing](https://en.wikipedia.org/wiki/Multiprocessing),
[Lock-Free Programming](https://preshing.com/20120612/an-introduction-to-lock-free-programming/),
[Cryptography](https://en.wikipedia.org/wiki/Cryptography)

---

### Introduction

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;Parallel programming is a paradigm of computer processing in which tasks or processes are broken down into smaller, independent units that can be executed simultaneously by the CPU. The goal of parallel programming is to leverage the processing power of multi-core CPUs, GPUs, or other computers or servers to improve the performance and speed of a program or computation. It is especially relevant in today's computing landscape, where heterogeneous and distributed systems are common.

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;Parallel programming is often used to tackle computationally intensive tasks such as data analysis, simulations, rendering, and more; by dividing the workload among multiple processing units. It can significantly reduce the time required to complete these tasks, making it a valuable skill for programmers and developers.

---

### Challenge Statements

**ELIGIBILITY: THESE CHALLENGE STATEMENTS CAN BE ATTEMPTED ONLY BY FIRST YEARS!**

The following rule is applicable to the challenge statements given below:

**You are permitted to use C or Rust programming language & their respective stdlibs, Linux syscall APIs and other libraries/APIs if they are specifically mentioned in the tasks and their resources. Use of any other languages/libraries will not be preferred.**

+ **Challenge I**:  
  Write a multi-threaded program in C (use `threads.h` & `stdatomic.h`) **or** Rust, to find the sum of all the elements in a randomly generated array of 64-bit unsigned integers, that has `N` elements where, `N` ≥ 1024. Use **only** 4 threads to compute the sum such that thread `i` (assuming `i` is the thread index), computes the sum of the elements from the index `i * N / 4` upto `(i + 1) * N / 4` in the array.

  Compare the amount of total time the multi-threaded program takes to finish, against the single-threaded case.
  
  This **must** be executed on a Linux machine.

+ **Challenge II**:  
  Youre provided with the following SHA-256 hashes:
    1. 32adca3bde67a61815896740ee954b0b3dea15f6851f1b35079698729081e2ba
    2. 4a8d2dcdcc90b1ee98bc16f7097163854f879654e44ec4ffc71e54afcf5d0c13
    
  Clues:
    + *Hash 1* was obtained from a string containing 7 characters (lowercase alphabets only).
    + *Hash 2* was obtained from a string containing 8 characters (mix of 3 uppercase & 5 lowercase alphabets).
  
  Using the clues write an optimized, multi-threaded program in C (use `threads.h` & `stdatomic.h`) **or** Rust to find the string by brute force, using all available CPU threads. Make use of openssl `libcrypto`. Use `openssl/sha.h` or `openssl/evp.h` headers in C or the `openssl-sys` crate in Rust to compute the hashes. Report the string your program finds.
  
  This **must** be executed on a Linux machine.
  
  PS: It might take several hours to find the string!
  
+ **Bonus**:  
  Cross compile the solution program of **Challenge I** so that you can execute it on an Android (aarch64) device via *Termux*.

**There are no hard requirements to fully complete these tasks. Take your time to understand the theory and its practical implications. Partial attempts shall be considered, but harder tasks will carry greater weightage.**

---

### Resources

+ **Common**:
  + [Linux man-pages](https://www.man7.org/linux/man-pages/)
  + [C Reference](https://en.cppreference.com/c)
  + [Rust std Docs](https://doc.rust-lang.org/std/)

+ **Challenge I**:
  + [Threads in Single-Core Systems (YouTube)](https://youtu.be/M9HHWFp84f0)
  + [Threads in Multi-Core Systems (YouTube)](https://youtu.be/5sw9XJokAqw)
  + [Synchronization Primitives (YouTube)](https://youtu.be/IMceN4_rieo)
  + [Atomic Operations](https://preshing.com/20130618/atomic-vs-non-atomic-operations/)
  + [C Reference for threads.h & stdatomic.h](https://en.cppreference.com/c/thread)
  + Rust [`thread`](https://doc.rust-lang.org/std/thread/index.html) & [`sync`](https://doc.rust-lang.org/std/sync/index.html) Docs
  + Rust [`atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html) Docs

+ **Challenge II**:
  + [Hash function](https://en.wikipedia.org/wiki/Hash_function)
  + [SHA-2](https://en.wikipedia.org/wiki/SHA-2)
  + [C Reference for threads.h & stdatomic.h](https://en.cppreference.com/c/thread)
  + Rust [`thread`](https://doc.rust-lang.org/std/thread/index.html) & [`atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html) Docs
  
+ **Bonus**:
  + [Android NDK download](https://developer.android.com/ndk/downloads)
  + [Termux](https://termux.dev/en/)

---

### Submission

Create a **private** GitHub repo and put all of your code/observations and a detailed README in there. Make sure to keep everything organized in different folders for each task. If possible, put up all the compiled programs in your releases page.

Add the mentors as collaborators to your repo once you're done.

---

### Mentor's Details

1. `Nimesh Acharya` (<+91 87624 21203>, GitHub: [Radonoxius](https://github.com/Radonoxius))

2. `Ranjit Tanneru` (<+91 81239 99357>, GitHub: [AmissDrake](https://github.com/AmissDrake))