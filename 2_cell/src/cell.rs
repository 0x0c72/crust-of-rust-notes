use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

// implied by UnsafeCell being in the struct
// impl<T> !Sync for Cell<T> {}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Cell {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        // SAFETY: we know noone else is concurrently mutating self.value (because !Sync)
        // SAFETY: we know we're not invalidating any references, becasue we never give any out
        unsafe { *self.value.get() = value };
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        // SAFETY: we know noone else is modifying this value, since only this thread can mutate
        // (because !Sync), and it is executing this function instead.
        unsafe { *self.value.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::Cell;
    use std::sync::Arc;
    use std::thread;

    // #[test]
    // fn bad() {
    //     let x = Arc::new(Cell::new([0; 1024]));
    //     let x1 = Arc::clone(&x);
    //     let jh1 = thread::spawn(move || {
    //         x1.set([1; 1024]);
    //     });
    //     let x2 = Arc::clone(&x);
    //     let jh2 = thread::spawn(move || {
    //         x2.set([2; 1024]);
    //     });

    //     jh1.join().unwrap();
    //     jh2.join().unwrap();
    //     let  xs = x.get();
    //     for &i in xs.iter() {
    //         eprintln!("{}",  i);
    //     }
        
    // }
}
