use crate::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;

// struct Foo<'a, T: Default> {
//     v: &'a mut T,
// }

// impl<T: Default> Drop for Foo<'_, T> {
//     fn drop(&mut self) {
//         std::mem::replace(self.v, T::default());
//     }
// }

// fn broken_main() {
//     let (foo, mut t);
//     t = String::from("hello");
//     foo = Rc::new(Foo { v: &mut t });
// }

struct SharedValue<T> {
    value: T,
    refcount: Cell<usize>,
}

pub struct Rc<T> {
    inner: NonNull<SharedValue<T>>, // not send because of `NonNull`
    _marker: PhantomData<SharedValue<T>>, // fix for drop check
}

impl<T> Rc<T> {
    fn new(v: T) -> Self {
        let inner = Box::new(SharedValue {
            value: v,
            refcount: Cell::new(1),
        });

        Rc {
            // using into_raw instead of dereferencing stops the box from being freed.
            // SAFETY: Box does not give us a null pointer
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        inner.refcount.set(c + 1);
        Rc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a `Box` that is only deallocated when the last `Rc` goes away
        // we have an `Rc`, therefore the `Box` has not been deallocated, so deref is fine.
        &unsafe { self.inner.as_ref() }.value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        if c == 1 {
            drop(inner);
            // SAFETY: we are the only `Rc` left, and we are being dropped
            // therefore after us, there will be no `Rc`s, and no references to `T`.
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            // there are otehr Rc's, so don't drop the Box!
            inner.refcount.set(c - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
