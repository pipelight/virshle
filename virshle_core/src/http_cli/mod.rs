/*
* Cloud hypervisor compatibility layer.
*
* This crate is an api to connect to socket and send http requests.
*
* Sources:
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
* https://github.com/amacal/etl0/blob/85d155b1cdf2f7962188cd8b8833442a1e6a1132/src/etl0/src/docker/http.rs
* https://docs.rs/hyperlocal/latest/hyperlocal/
*/

mod socket;
mod ssh;

pub use socket::UnixConnection;

// Error Handling
use virshle_error::{LibError, VirshleError, WrapError};

pub trait Connection<S>
where
    S: Sized,
{
    fn execute() -> Result<(), VirshleError>;
    fn get() -> Result<(), VirshleError>;
    fn post() -> Result<(), VirshleError>;
    fn put() -> Result<(), VirshleError>;
    fn open() -> Result<S, VirshleError>;
    fn close() -> Result<(), VirshleError>;
}
