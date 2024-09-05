use crate::error::CoreError;
use std::sync::{Mutex, MutexGuard};
use tracing::{debug, error};

pub(crate) trait MutexExt<T> {
    fn lock_sync(&self) -> Result<MutexGuard<'_, T>, CoreError>;

    #[allow(dead_code)]
    fn locking<R>(&self, f: impl FnOnce(MutexGuard<'_, T>) -> Result<R, CoreError>) -> Result<R, CoreError> {
        let guard = self.lock_sync()?;
        f(guard)
    }
}

impl<T> MutexExt<T> for Mutex<T> {
    #[inline(never)]
    #[track_caller]
    fn lock_sync(&self) -> Result<MutexGuard<'_, T>, CoreError> {
        let location = std::panic::Location::caller();
        match self.lock() {
            Ok(guard) => {
                debug!(
                    "Acquired lock. File: {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
                Ok(guard)
            }
            Err(err) => {
                let location = std::panic::Location::caller();
                error!(
                    "Failed to lock mutex: {}; file: {}:{}:{}",
                    err,
                    location.file(),
                    location.line(),
                    location.column()
                );
                Err(CoreError::MutexPoisoned(err.to_string()))
            }
        }
    }
}
