pub mod constants;
pub mod ctcp;
pub mod message;
pub mod model;
pub mod responses;

#[macro_export]
macro_rules! try_lock {
    ($locked: expr) => {
        match $locked.lock() {
            Ok(unlocked) => unlocked,
            Err(e) => {
                println!("[SERVER-MULTITHREAD] FATAL: Lock poisoned");
                println!("{}", e);
                e.into_inner()
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap {
    ($option: expr) => {
        match $option {
            Some(v) => v,
            None => {
                println!("Missing information");
                return Err(());
            }
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($text: expr) => {{
        println!("debug: {}", $text);
    }};
}

#[macro_export]
macro_rules! ignore {
    () => {};
}
