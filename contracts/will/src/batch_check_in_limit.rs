//! Upper bound on the number of will IDs accepted by `batch_check_in` (#414).

use soroban_sdk::{panic_with_error, Env};

use crate::errors::WillError;

/// Maximum number of will IDs a single `batch_check_in` call may process.
pub const MAX_BATCH_CHECK_IN: u32 = 50;

/// Panics with [`WillError::BatchTooLarge`] if `len` exceeds [`MAX_BATCH_CHECK_IN`].
pub fn assert_within_limit(env: &Env, len: u32) {
    if len > MAX_BATCH_CHECK_IN {
        panic_with_error!(env, WillError::BatchTooLarge);
    }
}
