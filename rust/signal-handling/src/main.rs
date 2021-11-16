use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use nix::sys::signal::{self, Signal as Sig};
use simple_signal::Signal;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    simple_signal::set_handler(&[Signal::Fpe], move |signals| {
        println!("{:?}", signals[0]);
        r.store(false, Ordering::SeqCst);
    });
    println!("Waiting for a signal...");
    while running.load(Ordering::SeqCst) {
        signal::raise(Sig::SIGFPE);
    }
    println!("Got it! Exiting...");
}