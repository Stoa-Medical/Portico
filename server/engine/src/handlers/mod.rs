pub mod fyi;
pub mod run;
pub mod sync;

// Re-export the modules as the old names to avoid changing too many imports
pub use self::run as command;
