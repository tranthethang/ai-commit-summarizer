use std::sync::Mutex;

// Global mutex to serialize tests that modify process-wide state (like current directory)
pub static TEST_MUTEX: Mutex<()> = Mutex::new(());
