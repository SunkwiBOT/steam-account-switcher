use std::sync::{Arc, Mutex};
use std::time::Instant;

use steam_core::authenticator::Authenticator;

use super::AppState;

impl AppState {
    pub fn authenticator(&self) -> Arc<Mutex<Authenticator>> {
        Arc::clone(&self.authenticator)
    }

    pub fn note_vault_unlocked(&self) {
        self.lock().unlocked_at = Some(Instant::now());
    }

    pub fn note_vault_locked(&self) {
        self.lock().unlocked_at = None;
    }

    /// Returns true only when the vault changed from unlocked to locked.
    pub fn lock_vault_if_due(&self) -> bool {
        let elapsed = {
            let inner = self.lock();
            let Some(unlocked_at) = inner.unlocked_at else {
                return false;
            };
            let elapsed = unlocked_at.elapsed().as_secs();
            if !inner
                .settings
                .data()
                .vault_lock_interval
                .locks_after(elapsed)
            {
                return false;
            }
            elapsed
        };
        let authenticator = self.authenticator();
        let locked = {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard.lock().is_ok()
        };
        if !locked {
            return false;
        }
        crate::logging::write(&format!(
            "action=lock_vault status=locked reason=idle duration={elapsed}s"
        ));
        self.lock().unlocked_at = None;
        true
    }
}
