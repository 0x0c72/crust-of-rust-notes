#![allow(dead_code)]

use std::{future::Future, net::TcpStream, process::Output};
use tokio;

async fn handle_connection(_: TcpStream) {
    let x = Arc::new(Mutex::new(vec![]));
    let x1 = Arc::clone(&x);
    let h = tokio::spawn(async move {
        deserialize()
    });
    h.await; // this 
    let handle = tokio::spawn(async move {
        x1.lock();
        let x: Result<_, _> = throws_error();
        // to get the error use something like tracing to emit it
    });
}


fn main() {
    // starts up multiple threads, each can await a future
    // only one future, so even with more threads there's only one future to await on
    let runtime = tokio::runtime::Runtime::new();
    runtime.block_on(async {

        let mut accept = tokio::net::TcpListener::bind("0.0.0.0:8080");
        while let Ok(stream) = accept.await {
            tokio::spawn(handle_connection(stream));
            // 
        }

        
        // tokio::spawn is not a thread spawn, it's just a task being stuck on the job queue, threads are spawned by the runtime
        
        println!("Hello, world!");

        let read_from_terminal = std::thread::spawn(move || { 
            let mut x = std::io::Stdin::lock(&std::io::stdin());
            for line in x.lines() {
                // do something on user input
            }
        });

        let read_from_network = std::thread::spawn(move || {
            let mut x = std::net::TcpListener::bind("0.0.0.0:8080").unwrap();
            while let Ok(stream) = x.accept() {
                // do something on stream
                let handle = std::thread::spawn(move || {
                    handle_connection(stream);
                });
            }
        });
        
        let mut network = read_from_network();
        let mut terminal = read_from_terminal();
        let mut foo = foo2();

        let mut f1 = tokio::fs::File::open("foo"); // need tokio versions for async functions (even though the file io itself is sync)
        let mut f1 = tokio::fs::File::create("bar");
        let copy = tokio::io::copy(&mut f1, &mut f2);

        loop {
            select! { // &mut allows you to reuse the values across the loops
                stream <- (&mut network).await => {
                    // do something on stream
                }
                line <- (&mut terminal).await => {
                    // do something with line
                    break;
                }
                foo <- (&mut foo).await => { // whoever has foo responsible for awaiting

                }
                _ <- (&mut copy).await => {
                    
                }
            } // be careful if one branch runs to completion but others don't

            // some bytes have been copied from foo to bar but not all
            // need to call copy.await again to finish it
            let files: Vec<_> = (0..3).map(|i| tokio::fs::read_to_string(format!("file{}", i))).collect();
            // this way - sequential approach
            let file1 = files[0].await;
            let file2 = files[1].await;
            let file3 = files[2].await;
            // or this way - concurrent approach
            let (file1, file2, file3) = join!(files[0], files[1], files[2]);
            // join runs them all concurrently, and when they're all done, gives you the output of them all
            // allows overlap of compute and io
            let file_bytes = join_all(files);
            assert!(file_bytes[0] == files[0]); 
            // output in same order as input, can opt out of ordering using something like FuturesUnordered 
            // join is smart enough to know to check only futures that have had events
            // select and join are NOT computing in parallel
        }
    });
}

async fn foo1() -> usize { 
    println!("foo");
    0 
}

async fn read_to_string(_: &str) {}

fn expensive_function(_: ()) {}

fn foo2(cancel: tokio::sync::mpsc::Receiver) -> impl Future<Output = usize> {
    async {
        read_to_string("file1").await;
        select! { // "race!"
            done <- read_to_string("file2").await => {
                // continue; or fall thro to println below
            }
            cancel <- cancel.await => {
                // return 0;
            } 
        };
        expensive_function(x);
        read_to_string("file3").await;
    };
    async { 
        println!("foo1");
        await = let fut = foo1();
        while !fut.is_ready() {
            std::thread::yield_now();
            fut.try_complete();
        }
        let result = fut.take_result();
        foo2().await;
        println!("foo2");
        

        async {
            let fut = read_to_string("file");
            let x  = loop {
                if let Some(result) = fut.try_check_completed() {
                    break result;
                } else {
                    fut.try_make_progress();
                    yield;
                }
            }
        }
    }
}

enum StateMachine {
    Chunk1 { x: [u8; 1024], fut: tokio::fs::ReadIntoFuture<'x> },
    Chunk2 {},
}



fn other_main() {

}

fn mn() { println!("d");}

async fn foo() {
    {
        let mut x = [0; 1024];
        let z = vec![];
        let fut = tokio::fs::read_into("file.dat", &mut x[..]);
    }
    // fut.await;
    yield; // really: return
    
    { // chunk 2
        let n = fut.output();
        println!("{:?}", x[..n]);
    }
}

struct Request;
struct Response;

trait Service<Request> {
    type CallFuture: Future<Output = Response>;
    fn call(&mut self, _: Request) -> Self::CallFuture;
    //fn call(&mut self, _: Request) -> impl Future<Output = Response>;
}

struct X;

impl Service for X {
    type CallFuture = Pin<Box<dyn Future<Output = Response>>>;
    fn call(&mut self, _: Request) -> Self::CallFuture {
        Box::pin(async move { Response })
        // async { Response } 
    }
}

struct Y;

impl Service for Y {
    type CallFuture = Pin<Box<dyn Future<Output = Response>>>;
    fn call(&mut self, _: Request) -> Self::CallFuture {
        let z = [0; 1024];
        tokio::time::sleep(100).await;
        drop(z);
        Box::pin(async move { Response })
    }
}

struct FooCall<F>(F);

fn foo(<S: Service>x: S) -> FooCall<typeof S::Call> {
    let fut = x.call(Request);
    FooCall(fut)
}