use std::time::Instant;

mod future;
mod http;

use future::{Future, PollState};
use http::Http;

struct Coroutine {
    state: State,
}

/*
.await就代表了各个状态
*/

enum State {
    /// has been created but not polled
    Start,
    /// first call Http::get and get a new HttpGetFuture
    Wait1(Box<dyn Future<Output = String>>),
    /// second call Http::get and get a new HttpGetFuture
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");
                    // 每得到一个future 就对应一个状态
                    let fut = Box::new(Http::get("/600/HelloWorld1"));
                    self.state = State::Wait1(fut);
                }
                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/400/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut) => match fut.poll() {
                    PollState::Ready(txt2) => {
                        println!("{txt2}");
                        self.state = State::Resolved;
                        break PollState::Ready(());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}

fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

fn main() {
    let mut future = async_main();
    loop {
        match future.poll() {
            PollState::NotReady => {
                println!("Schedule other tasks");
                // 模拟执行其他任务
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            PollState::Ready(_) => break,
        }
    }
}
