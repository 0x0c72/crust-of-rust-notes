use anyhow;
use clap::{AppSettings, Clap};
use colour::{red_ln, green_ln, blue_ln, red, green, blue, dark_yellow, dark_yellow_ln, cyan, cyan_ln};
use crossbeam::{channel::unbounded, select};
use futures::{future, Future};
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::slice::SliceIndex;
use thiserror::Error;
use tokio::io::{stdin, AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader, BufStream, Stdin};
use tokio::signal;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;
use tokio::{
    self,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use tower::Service;
// use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

struct MyWatcher;

type Event = DebouncedEvent;
type EventStream = Receiver<Event>;

#[derive(Clap)]
struct Opts {
    #[clap(short)]
    verbose: bool,
}

struct WatchSettings {
    paths: Vec<PathBuf>,
    mode: RecursiveMode,
    delay: tokio::time::Duration,
}

impl Service<WatchSettings> for MyWatcher {
    type Response = EventStream;

    type Error = ();

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: WatchSettings) -> Self::Future {
        todo!()
    }
}

#[derive(Error, Debug)]
enum MyError {
    #[error(transparent)]
    InputError(#[from] anyhow::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
}

pub type BoxError = std::boxed::Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    if opts.verbose {
        println!("Running with verbose flag");
    } else {
        println!("Running without verbose flag");
    }

    let (txw, mut rxw): (std::sync::mpsc::Sender<Event>, std::sync::mpsc::Receiver<Event>) = std::sync::mpsc::channel();
    let (tx_stdin, mut rx_stdin): (
        std::sync::mpsc::Sender<Vec<PathBuf>>,
        std::sync::mpsc::Receiver<Vec<PathBuf>>,
    ) = std::sync::mpsc::channel();
    let (shutdown_send, mut shutdown_recv): (
        tokio::sync::mpsc::UnboundedSender<()>,
        tokio::sync::mpsc::UnboundedReceiver<()>,
    ) = tokio::sync::mpsc::unbounded_channel();
    let (shutdown_notify_send, mut shutdown_notify_recv): (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ) = tokio::sync::broadcast::channel(32);
    let tx_stdin2 = tx_stdin.clone();
    let stdin_handle = tokio::spawn(async move {
        let x = get_input(">").await; //.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync + 'static>)?;
        let v = x
            .into_iter()
            .map(|x| {
                println!("Got {:?} from stdin", &x);
                x.into()
            })
            .collect::<Vec<PathBuf>>();
        tx_stdin2.send(v).unwrap();
        println!("done stdin");
        drop(tx_stdin2);
    });
    
    let txw2 = txw.clone();
    let watch_handle = tokio::spawn(async move {
        let mut watcher: RecommendedWatcher = Watcher::new(txw2, Duration::from_secs(1)).unwrap();
        while let Ok(v) = rx_stdin.recv() {
            for path in v {
                let path = format_path(path);
                println!("Got {:?} to watch", &path);
                // spawns std::thread
                watcher.watch(Path::new(&path), RecursiveMode::Recursive).unwrap();
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (txw_tokio, mut rxw_tokio) = crossbeam::channel::unbounded();
    let txw_tokio2 = txw_tokio.clone();
    let shutdown_notify_send_other = shutdown_notify_send.clone();
    let other_handle = tokio::spawn(async move {

        while let Ok(x) = rxw.recv() {
            if opts.verbose {
                match &x {
                    DebouncedEvent::Create(p) => {
                        green!("[Info] ");
                        cyan!("CREATE -> ");
                        println!("{:?}", p);                    
                    }
                    DebouncedEvent::Chmod(p) => {
                        green!("[Info] ");
                        cyan!("CHMOD -> ");
                        println!("{:?}", p);                    
                    }
                    DebouncedEvent::Write(p) => {
                        green!("[Info] ");
                        cyan!("WRITE -> ");
                        println!("{:?}", p);
                    }
                    DebouncedEvent::Remove(p) => {
                        green!("[Info] ");
                        cyan!("REMOVE -> ");
                        println!("{:?}", p);
                    }
                    DebouncedEvent::Rename(p, p2) => {
                        green!("[Info] ");
                        cyan!("RENAME -> ");
                        print!("{:?}", p);
                        cyan!(" to ");
                        println!("{:?}", p2);
                    }
                    DebouncedEvent::Rescan => {
                        dark_yellow!("[Warn] ");
                        println!("RESCAN");
                    }
                    DebouncedEvent::NoticeRemove(p) => {
                        green!("[Info] ");
                        cyan!("REMOVE NOTICE -> ");
                        println!("{:?}", p);
                    }
                    DebouncedEvent::NoticeWrite(p) => {
                        green!("[Info] ");
                        cyan!("WRITE NOTICE -> ");
                        println!("{:?}", p);
                    }
                    DebouncedEvent::Error(e, p) => {
                        if let Some(path) = p {
                            red!("[Error] ");
                            dark_yellow!("Path ");
                            print!("{:?}", path);
                            dark_yellow!(": ");
                            println!("{:?}", e);
                        } else {
                            red!("[Error] ");
                            dark_yellow!("Path Unknown: ");
                            println!("{:?}", e);
                        }
                    }
                    _ => {}
                }
            } else {
                green!("[CONVERTER] ");
                println!("{:?}", x);
            }
            let x = format!("{:?}", x);
            txw_tokio2.send(x).unwrap();
        }
    });

    let select_handle = tokio::spawn(async move {
        let mut count = 0;
        loop {
            // tokio loop
            // tokio::select! {
            //     v = rxw_tokio.recv() => {
            //         println!("[In {}] Got {:?} from watcher", &count, v);
            //         count += 1;
            //     },
            //     else => {
            //         println!("All channels closed");
            //         break;
            //     }
            // }

            crossbeam::select! {
                recv(rxw_tokio) -> msg => {
                    println!("Got {:?} from watcher", msg);
                },
                default => {},
            }
            tokio::select! {
                _ = signal::ctrl_c() => {
                    println!("got ctrl_c");
                    shutdown_notify_send.send(()).unwrap();
                    std::process::exit(1);
                },
                _ = shutdown_recv.recv() => {
                    println!("shutdown signal received");
                    break;
                },
            }
        }
    });

    // drop(tx_stdin);
    // drop(txw);
    // drop(rxw_tokio);

    stdin_handle.await.unwrap();
    watch_handle.await.unwrap();
    other_handle.await.unwrap();
    select_handle.await.unwrap();
}

async fn talking_threads() -> Result<(), BoxError> {
    let strings = vec![
        "hello 1", "hello 2", "hello 3", "hello 4", "hello 5", "hello 6", "hello 7", "hello 8", "hello 9", "hello 10",
    ];
    let (tx, mut rx) = crossbeam::channel::unbounded();
    let (tx2, mut rx2) = crossbeam::channel::unbounded();
    let first_tx = tx.clone();
    let first_tx2 = tx2.clone();
    let first_handle = tokio::spawn(async move {
        loop {
            let mut rng = rand::thread_rng();
            let rand = rng.gen_range(0..10);
            println!("START -> {}", strings[rand]);
            first_tx.send(strings[rand]).unwrap();
            let _ = tokio::time::sleep(Duration::from_secs(rand as u64)).deadline();
            if let Ok(x) = rx2.recv() {
                let msg = match x {
                    "1" => "1!!!!!!",
                    "2" => "2!!!!!!",
                    "3" => "3!!!!!!",
                    "4" => "4!!!!!!",
                    "5" => "5!!!!!!",
                    "6" => "6!!!!!!",
                    "7" => "7!!!!!!",
                    "8" => "8!!!!!!",
                    "9" => "9!!!!!!",
                    "10" => "10!!!!!!",
                    _ => "?!!!!!!",
                };
                println!("END -> {}", msg);
            }
        }
    });

    let second_tx = tx.clone();
    let second_tx2 = tx2.clone();
    let second_handle = tokio::spawn(async move {
        loop {
            if let Ok(x) = rx.recv() {
                println!("FIRST -> {}", x);
                let msg = match x {
                    "hello 1" => "1",
                    "hello 2" => "2",
                    "hello 3" => "3",
                    "hello 4" => "4",
                    "hello 5" => "5",
                    "hello 6" => "6",
                    "hello 7" => "7",
                    "hello 8" => "8",
                    "hello 9" => "9",
                    "hello 10" => "10",
                    _ => "?",
                };
                println!("SECOND -> {}", msg);
                second_tx2.send(msg).unwrap();
            }
        }
    });

    first_handle.await.unwrap();
    second_handle.await.unwrap();
    Ok(())
}

async fn get_input(prompt: &str) -> Vec<String> {
    let mut lines = Vec::new();
    println!("Enter a directory path to watch:");
    let _ = tokio::io::stdout().flush();    
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buf = String::new();
    while let Ok(n) = reader.read_line(&mut buf).await {
        match n {
            0 => break,
            _ => {
                lines.push(buf.clone().replace("\n", ""));
                if buf.ends_with("\n") {
                    break;
                }
            }
        }
    }
    lines
}

fn format_path(s: PathBuf) -> PathBuf {
    let s = s.display().to_string();
    let s = s.replace("\r", "");
    let s = s.replace("\n", "");
    let s = s.replace("\t", "");
    s.into()
}

mod command {
    use super::*;
    type FnPtr<T, U> = fn(T) -> U;

    pub trait Events {
        fn on_error(&self, err: &str) {}
        fn on_shutdown(&self) {}
    }

    pub trait Cmd {
        fn execute(&self) -> Result<(), BoxError>;
    }

    struct Message {
        commands: Vec<Box<dyn Cmd>>,
    }

    impl Message {
        fn new() -> Self {
            Self { commands: Vec::new() }
        }

        fn append_cmd(&mut self, cmd: Box<dyn Cmd>) {
            self.commands.push(cmd);
        }

        fn undo(&mut self) {
            self.commands.pop();
        }

        fn clear(&mut self) {
            self.commands.clear();
        }

        fn execute(&self) -> Result<(), BoxError> {
            self.commands.iter().map(|cmd| cmd.execute());
            Ok(())
        }
    }

    impl Cmd for Message {
        fn execute(&self) -> Result<(), BoxError> {
            for command in &self.commands {
                command.execute();
            }
            Ok(())
        }
    }

    struct TaskBlock {
        messages: Vec<Message>,
        handle: Option<JoinHandle<()>>,
        hooks: Vec<Box<dyn Events>>,
    }

    impl TaskBlock {
        fn new() -> Self {
            Self {
                messages: vec![],
                handle: None,
                hooks: vec![],
            }
        }

        fn add_message(&mut self, msg: Message) {
            self.messages.push(msg);
        }

        fn execute(&self) -> Result<(), BoxError> {
            for msg in &self.messages {
                for command in &msg.commands {
                    command.execute().unwrap();
                }
            }
            Ok(())
        }
    }

    struct MyCommand;

    impl Cmd for MyCommand {
        fn execute(&self) -> Result<(), BoxError> {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_talking_threads() {
            todo!();
        }
    }
}