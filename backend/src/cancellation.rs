use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

const IDLE: u8 = 0;
const RUNNING: u8 = 1;
const CANCELLED: u8 = 2;

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicU8>);

pub struct Operation(Cancellation);

impl Drop for Operation {
    fn drop(&mut self) {
        self.0.0.store(IDLE, Ordering::SeqCst);
    }
}

impl Cancellation {
    pub fn begin(&self) -> Operation {
        self.0.store(RUNNING, Ordering::SeqCst);
        Operation(self.clone())
    }
    pub fn request(&self) {
        let _ = self
            .0
            .compare_exchange(RUNNING, CANCELLED, Ordering::SeqCst, Ordering::SeqCst);
    }
    pub fn requested(&self) -> bool {
        self.0.load(Ordering::SeqCst) == CANCELLED
    }
    pub fn check(&self) -> anyhow::Result<()> {
        if self.requested() {
            return Err(Cancelled.into());
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&crate::i18n::message("notice.exportCancelled"))
    }
}

impl std::error::Error for Cancelled {}
