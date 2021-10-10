// strtok

pub fn strtok<'a, 'b>(s: &'a mut &'b str, delimiter: char) -> &'b str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        let suffix = &s[(i + delimiter.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        *s = "";
        prefix
    }
}

fn main() {
    let s = String::new();
    let x: &'static str = "hello world";
    let mut y/* :  &'a */ = &*s;
    y = x; // 'static -> 'a
}

// T: U
// T is at least as useful as U
//
// 'static: 'a
//
// class Animal;
// class Cat: Animal;
//
// Cat: Animal
//
// can do as much with Cat as Animal and so on
//
// covariance
//  fn foo(&' str) {}
//  let x: &'a str
//
//  foo(&'a str)
//  foo(&'static str) // 'static is a subtype of 'a
//
//  x = &'a str
//  x = &'static str
//
// why isn't every type covariant?
//
//
// contravariance
//
//  fn foo(bar: Fn(&'a str) -> ()) {
//      bar("" /* 'a */) // wouldn't compile, requires static
//      bar("hello world") // 'static str, more useful will compile
//  }
//  let x: Fn(&'a str) -> ()
//
//  foo(fn(&'static str) {}) // (invalid syntax) requires static here; caller has to give a MORE useful/stricter type ('static)
//  foo(fn(&'a str) {}) // (invalid syntax) more useful will compile
//
// variance requirement is flipped when it comes to function arguments
//
//  &'static str // more useful
//  &'a str // less useful
//
//  'static <: 'a (subtype of any other lifetime)
//  &'static T <: &'a T
//
//  Fn(&'static str) // less useful
//  Fn(&'a str) // more useful (less strict)
//
//  'static <: 'a
//  Fn(&'a T) <: Fn(&'static T)
//
// flipped for functions (contravariant)
//
// invariance
//
//  fn foo(&mut &'a str, x: &'a str) {
//      *s =  x;
//  }
//
//  let mut x: &'static str = "hello world";
//  let z = String::new();
//  foo(&mut x, &z);
//  // foo(&mut &'a      str, &'a str) must be equal when behind mutable reference
//  // foo(&mut &'static str, &'a str)
//  drop(z);
//  println!("{}", x); // x pointing to dropped stacklocal memory
//
// if we didn't have invariance for mutable references, you could downgrade a mutable reference but outside the fn, it wouldn't know that it's less useful now
//
//
// it's okay to shorten the lifetime of a mutable borrow
//  fn bar() {
//      let mut y = true;
//      let mut z /* : &'y mut bool */= &mut y;
//
//      let x = Box::new(true);
//      let x: &'static mut bool = Box::leak(x);
//
//      let _ = z; // ignore this line
//
//      z = x; // &'y mut bool = &'static mut bool - lifetime shortened here
//
//      drop(z); // ignore this line
//  }
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut x = "hello world";
        // strtok<'a, 'b>(&'a mut &'b str) -> &'b str
        // strtok(&'a mut &'static str) -> &'b str <- will replace all the b with static
        let z = &mut x;
        // borrow of x stops here since z is not used after this point, mut are covariant over lifetimes
        let hello = strtok(&mut x, ' ');
        assert_eq!(hello, "hello");
        assert_eq!(x, "world"); // as long as hello lives, x is mutably borrowed
    }

    #[test]
    fn it_works2() {
        fn check_is_static(_: &'static str) {}

        let mut x = "hello world";
        check_is_static(x);

        // <'a> &'a mut &'a str
        //      &'x mut &'static str // can't shorten lifetime of what's behind mut, bc they're invariant in arguments

        strtok(&mut x, ' ');
        assert_eq!(x, "world");
    }
}
