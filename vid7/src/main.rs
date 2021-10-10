#![feature(dropck_eyepatch)]

use std::default;
use std::iter::Empty;
use std::marker::PhantomData; // can be generic over another type but doesn't contain it
use std::ptr::NonNull;

pub struct Boks<T> {
    p: NonNull<T>,
    _t: PhantomData<T>, // tells compiler we will drop the T (needed when using may_dangle)
                        // _t: PhantomData<fn() -> T>, // makes covariant over T, but no longer subject to drop check
}

// Example
struct Deserializer<T> {
    // _t: PhantomData<T>, // doesns't contain a T, but means the drop check (check if fields read during drop) is applied to the T of the Deserializer<T>
    // instead..
    _t: PhantomData<fn() -> T>, // creates covariance, and wont apply the drop check on T

                                // _t: PhantomData<fn(T)>, // not this, this is contravariant
                                // _t: PhantomData<fn(T) -> T>, // not this, this is invariant
}
// Deserializer<Oisann<i32>>;

struct EmptyIterator<T> {
    _t: PhantomData<fn() -> T>, // creates covariance, and wont apply the drop check on T
                                // _t: PhantomData<T>, // also covariant, but triggers drop check for T
}
impl<T> Iterator for EmptyIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<T> Default for EmptyIterator<T> {
    fn default() -> Self {
        EmptyIterator { _t: PhantomData }
    }
}

impl<T> Boks<T> {
    pub fn ny(t: T) -> Self {
        Boks {
            // SAFETY: Box never creates a null pointer
            p: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(t))) },
            _t: PhantomData,
        }
    }
}

// if generic type, compiler assumes that dropping T uses a T whether it does or not
// eyepatch lets us mask a paremeter from drop check "#[may_dangle]"
unsafe impl<#[may_dangle] T> Drop for Boks<T> {
    fn drop(&mut self) {
        // let _: u8 = unsafe { std::ptr::read(self.p as *const u8) };

        // SAFETY: p was constructed from a Boks in the first place and has not been freed
        // otherwise, since self still exists (othaerwise drop could not be called)
        unsafe { Box::from_raw(self.p.as_mut()) };
    }
}

impl<T> std::ops::Deref for Boks<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: is valid since it was constructed from a valid T,
        // and turned into a pointer through Box which creates aligned pointers,
        // and hasn't been freed since Self is alive
        unsafe { &*self.p.as_ref() }
    }
}

impl<T> std::ops::DerefMut for Boks<T> {
    // DerefMut is subtrait of Deref so compiler knows what Self::Target is
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: is valid since it was constructed from a valid T,
        // and turned into a pointer through Box which creates aligned pointers,
        // and hasn't been freed since Self is alive, also since we have a &mut self
        // no other reference has been given out to p
        unsafe { &mut *self.p.as_mut() }
    }
}

use std::fmt::Debug;
struct Oisann<T: Debug>(T);

// impl<T: Debug> Drop for Oisann<T> {
//     fn drop(&mut self) {
//         println!("{:?}", self.0);
//     }
// }

fn main() {
    let x = 42;
    let b = Boks::ny(x);
    println!("{:?}", *b);

    let mut y = 42;
    let b = Boks::ny(&mut y);
    // let b2 = Box::new(&mut y); // compiler knows that the Box won't read what it contains
    println!("{:?}", y); // dropck_eyepatch makes this work

    let mut z = 42;
    let b = Boks::ny(Oisann(&mut z));
    println!("{:?}", z);
    // drop(b) accesses mutable reference because of `Oisann<T>(T)`'s drop implementation

    let zz = String::new();
    let mut b = Boks::ny(&*zz);
    let zzz: &'static str = "hello world";
    check_static(zzz);
    b = Boks::ny(zzz);

    let s = String::from("hei");
    let mut _boks1 = Boks::ny(&*s);
    let boks2: Boks<&'static str> = Boks::ny("heisann");
    _boks1 = boks2; // compiler knows it's allowed to shorten the lifetime

    let s = String::from("hei");
    let mut _box1 = Box::new(&*s);
    let box2: Box<&'static str> = Box::new("heisann"); // cannot overwrite with a Box with longer lifetime because invariant over T
    _box1 = box2; // compiler knows it's allowed to shorten the lifetime

    let mut a = 42;
    let mut it: EmptyIterator<Oisann<& /* 'a */ mut i32>> = EmptyIterator::default(); // wouldn't compile if Empty implemented Drop
                                                                                      // struct Empty(PhantomData<T>);
                                                                                      // Empty<Oisann<&mut a>>
    let mut o = Some(Oisann(&mut a));
    {
        o = it.next();
    }
    drop(o); // 'a ends here
    println!("{:?}", a);
    // drop(it);
}

fn check_static(_: &'static str) {}

// Because of the invariance:
// cannot treat Box<& 'static str> as Box<&'some_shorter_lifetime str>
// even though &'static str as &'some_shorter_lifetime str
// and can treat Box<&'static str> as Box<&'some_shorter_lifetime str>
