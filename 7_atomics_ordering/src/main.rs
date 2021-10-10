use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

const LOCKED: bool = true;
const UNLOCKED: bool = false;

struct Mutex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>,
}

// opt into Sync because we know UnsafeCell will be thread-safe in this
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }
    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            // compare_exchange is expensive
            .compare_exchange_weak( 
                UNLOCKED,
                LOCKED, 
                Ordering::Relaxed, 
                Ordering::Relaxed
                )
            .is_err()
        {
            // research - MESI protocol
            // a given cache line can either be shared or exclusive (or some other states)
            // in compare_exchange() exclusive access is required
            // multiple threads can have a value in shared state at the same time
            // will often see a second, inner loop:
            // if we fail to take the lock we're just going to spin and just read the value
            // to keep lock in shared state 
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::thread::yield_now();
            }
            std::thread::yield_now();
            // maybe another thread runs here - race
            // self.locked.store(LOCKED, Ordering::Relaxed);
    
            // x86: CAS (compare and swap)
            // ARM: LDREX STREX (load/store exclusive)
            // - compare_exchange: impl using a loop of LDREX and STREX use when not called in loop
            // - compare_Exchange_weak: impl using LDREX STREX directly (on x86_64 it's a compare and swap) - use when calling in a loop
        }
        
        // SAFETY: we hold the lock, therefore we can create a mutable reference
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret
    }
}

use std::thread::spawn;
fn main() {
    let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));
    let handles: Vec<_> = (0..100) // hardcore functional rust way of range based for loop
        .map(|_| {
            spawn(move || {
                for _ in 0..1000 {
                    l.with_lock(|v| *v += 1);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(l.with_lock(|v| *v), 100 * 1000);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}