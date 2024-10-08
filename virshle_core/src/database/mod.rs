//!
//! The main/core struct to be manipulated
//!
//! Jucenit uses a kind of main store struct that eases the generation of
//! an nginx-unit Json configuration.
//!
//! This is a powerful intermediate
//! that is, in the end, lossy converted to a nginx-unit configuration.
//!

mod crud;
pub mod entity;

// Reexports
// pub use crud::*;
pub use crud::{connect_db, fresh_db};
pub use entity::*;
