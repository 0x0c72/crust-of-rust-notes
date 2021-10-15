use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};
use std::thread::JoinHandle;
use std::borrow::{Borrow, BorrowMut};

struct MyStruct {
    x: i32,
    y: i32,
    z: Mutex<i32>,
    v: Option<Mutex<u8>>,
}

impl  MyStruct {
    fn new() -> Self {
        MyStruct {
            x: 0,
            y: 0,
            z: Mutex::new(0),
            v: None,
        }
    }
    
    fn print_self(&self) {
        println!("x: {}", self.x);
        println!("y: {}", self.y);
    }

    fn n(&mut self) {
        self.x = 6;
    }
    
    fn o(&mut self) {
        self.x = 7;
    }
    
    fn r(self: &Arc<Self>) {
        let t = self.clone();
        println!("{:?}", t.x);
        
        println!("{:?}", t.x);
    }

    fn spawn_worker(self: Arc<Self>) -> Arc<Self> {
        let mut self_ref = self.clone();
        let p = self_ref.clone();
        let p2 = p.z.lock().unwrap();
        *self_ref.z.lock().unwrap() = 1;
        *self_ref.v.as_ref().unwrap().lock().unwrap() = 2;
        let r = self_ref.borrow_mut();
        self
    }

    fn s(&self) {
        println!("SSSS");
    }
}




fn main() {
    let mut a = MyStruct::new();
    a.n();
    a.print_self();
    a.o();
    a.print_self();
    let b = Arc::new(a);
    b.r();
    let c = b.clone();
    c.r();
    let my_struct = Arc::new(MyStruct::new()).spawn_worker();
    let mut x = Arc::new(Mutex::new(0));
    let r = x.borrow_mut();
    let x2 = x.clone();
    *x2.lock().unwrap() = 1;
}