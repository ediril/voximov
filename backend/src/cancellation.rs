use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[derive(Default)]
struct State {
    running: Option<String>,
    cancelled: bool,
    early: Option<String>,
}

#[derive(Clone, Default)]
pub struct Cancellation(Arc<Mutex<State>>);

pub struct Operation(Cancellation);

impl Drop for Operation {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.running = None;
        state.cancelled = false;
    }
}

impl Cancellation {
    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
    pub fn begin(&self, ticket: &str) -> Operation {
        let mut state = self.state();
        state.cancelled = state.early.take().as_deref() == Some(ticket);
        state.running = Some(ticket.to_owned());
        Operation(self.clone())
    }
    pub fn request(&self, ticket: &str) {
        let mut state = self.state();
        if state.running.as_deref() == Some(ticket) {
            state.cancelled = true;
        } else {
            state.early = Some(ticket.to_owned());
        }
    }
    pub fn requested(&self) -> bool {
        let state = self.state();
        state.running.is_some() && state.cancelled
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
