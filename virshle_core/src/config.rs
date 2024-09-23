// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub static MANAGED_DIR: Lazy<Arc<Mutex<String>>> =
    Lazy::new(|| Arc::new(Mutex::new("/var/lib/virshle/".to_owned())));
