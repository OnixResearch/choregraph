// Negative boundary fixture. The epoch-fence core must not read shared mutable state.
use std::sync::Mutex;

static SHARED_CURRENT_EPOCH: Mutex<u64> = Mutex::new(0);

pub fn read_shared_epoch() -> u64 {
    *SHARED_CURRENT_EPOCH.lock().expect("negative fixture mutex must lock")
}
