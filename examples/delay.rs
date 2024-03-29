use futures::future::poll_fn;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

struct Delay {
    when: Instant,
    // This Some when we have spawned a thread, and None otherwise.
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // First, if this is the first time the future is called, spawn the
        // timer thread. If the timer thread is already running, ensure the
        // stored `Waker` matches the current task's waker.
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            // Check if the stored waker matches the current task's waker.
            // This is necessary as the `Delay` future instance may move to
            // a different task between calls to `poll`. If this happens, the
            // waker contained by the given `Context` will differ and we
            // must update our stored waker to reflect this change.
            if !waker.will_wake(cx.waker()) {
                println!("{}", "换了waker");
                *waker = cx.waker().clone();

            }
        } else {
            println!("55555",);
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            // This is the first time `poll` is called, spawn the timer thread.
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                // The duration has elapsed. Notify the caller by invoking
                // the waker.
                let waker = waker.lock().unwrap();//当多个task调用同一个poll时候 必须保证再最内层的waker上调用wake
                println!("延时结束 调用wake",);
                waker.wake_by_ref();
            });
        }
        println!("44444",);
        // Once the waker is stored and the timer thread is started, it is
        // time to check if the delay has completed. This is done by
        // checking the current instant. If the duration has elapsed, then
        // the future has completed and `Poll::Ready` is returned.
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            // The duration has not elapsed, the future has not completed so
            // return `Poll::Pending`.
            //
            // The `Future` trait contract requires that when `Pending` is
            // returned, the future ensures that the given waker is signalled
            // once the future should be polled again. In our case, by
            // returning `Pending` here, we are promising that we will
            // invoke the given waker included in the `Context` argument
            // once the requested duration has elapsed. We ensure this by
            // spawning the timer thread above.
            //
            // If we forget to invoke the waker, the task will hang
            // indefinitely.
            Poll::Pending
        }
    }
}
#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_secs(2);
    let mut delay = Some(Delay { when, waker: None });

    poll_fn(move |cx| {
        let mut delay = delay.take().unwrap();
        let res = Pin::new(&mut delay).poll(cx); //waker1
        assert!(res.is_pending());
        tokio::spawn(async move {
            println!("{}", 22222);
            delay.await; //waker2
            println!("{}", 33333);
        });
        Poll::Ready(())
    })
    .await;
    println!("{}", 123123);
    sleep(Duration::from_secs(20));
}
