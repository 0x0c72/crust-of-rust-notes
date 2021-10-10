use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

// Channel Flavors:
// Synchronous: send can block, (bounded) limited capacity,
//  - Mutex + Condvar: VecDeque have sender block if full
//  - Atomic VecDeque/queue: head/tail pointers updated atomically, thread::park + thread::Thread::notify primitive - system for waking up blocking channels
// Asynchronous: send cannot block, unbounded capcity
//  - Mutex + Condvar + VecDeque: what we made here
//  - Mutex + Condvar + LinkedList (to prevent resizing): sender appends to list, receiver takes the head and walks backwards, only need tail pointer
//  - Atomic Queue/LinkedList: linked list of T
//  - Atomic Block LinkedList (crossbeam): linked list of atomic VecDeqeue<T> to prevent one sender blocking when two try to send and update LL tail pointer at the same time
// Rendezvous: synchronous channel with capacity = 0, used to
//             synchronize two threads rather than for sending 
//             values, type is often ()
// Oneshot Channels: any capacity, only one call to send - can atomically swap in and out of a place in memory that's either Some or None

// async/await
// hard to write one that works for both futures and blocking as futures want to return when waking up
// different sync primitives
// hard to write an implementation that internally knows whether it's being used in async futures context or blocking channels context without exposing to user

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

// need to implement manually so that the Clone trait bound is not there
// we don't need it since we are using an Arc
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);

        Sender {
            // use Arc::clone() to explicitly say you want to clone the Arc and not the thing inside of it,
            // the dot operator deref coercion would call wrong clone method
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner);
        if was_last {
            self.shared.available.notify_one();
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, val: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(val);
        drop(inner); // drop lock so the thing being notified can wake up and use it right away
                     // Notify that it's time to wake up as there is work to be done
        self.shared.available.notify_one(); // doesn't notify specific threads, can be any thread
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>, // optimization don't need to take lock for every receive
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t);
        }

        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(t) => {
                    // optimization to not need to take the lock to receive every send one by one
                    if !inner.queue.is_empty() {
                        std::mem::swap(&mut self.buffer, &mut inner.queue); // take everything that has been sent at once
                    }
                    return Some(t);
                }
                // None if Arc::strong_count(&self.shared) == 1 => return None, // make sure the arc rc is 1 meaning there's no senders (since the 1 is the recveiver)
                None if inner.senders == 0 => return None, 
                None => {
                    // thread goes to sleep until condvar wakes it up
                    // OS doesn't guarentee you aren't woken up with no work
                    // can be woken up for no reason
                    inner = self.shared.available.wait(inner).unwrap();
                }
            }
        }
    }
}

struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
}

struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

pub fn _channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::default(),
        senders: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        available: Condvar::default(),
    };
    let shared = Arc::new(shared);
    (Sender { shared: shared.clone() }, Receiver { shared: shared.clone(), buffer: VecDeque::default() })
}

// Example of an iterator over `Receiver<T>`
// impl<T> Iterator for Receiver<T> {
//     type Item = T;
//     fn next(&mut self) -> Option<Self::Item> {
//         // since we're using Option on the recv method this is trivial
//         self.recv()
//     }
// }

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn ping_pong() {
        let (mut tx, mut rx) = _channel();
        tx.send(42);
        assert_eq!(rx.recv(), Some(42));
    }

    #[test]
    fn closed_tx() {
        let (tx, mut rx) = _channel::<()>();
        drop(tx);
        assert_eq!(rx.recv(), None);
    }

    #[test]
    fn closed_rx() {
        let (mut tx, rx) = _channel();
        drop(rx);
        tx.send(42);
    }
}
