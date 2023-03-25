//! Synchronization object for throttling receives on client/server
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

/// A Synchronization object for throttling receives on client/server
pub struct Throttle {
    /// Receives continue when the throttle is open
    open: AtomicBool,
    /// Notifies the receive task it can resume
    notify: Notify,
}

impl Throttle {
    pub fn new() -> Throttle {
        Throttle {
            open: AtomicBool::new(true),
            notify: Notify::new(),
        }
    }

    /// Open the throttle, wakes the receive thread if throttle was closed
    pub fn open(&self) {
        if self.open.swap(true, Ordering::AcqRel) == false {
            self.notify.notify_one();
        }
    }

    /// Close the throttle, receiver will block until open
    pub fn close(&self) {
        self.open.swap(false, Ordering::Release);
    }

    /// Check and wait for the throttle to be opened
    pub async fn check(&self) -> () {
        while !self.open.load(Ordering::Acquire) {
            self.notify.notified().await;
        }
    }
}
