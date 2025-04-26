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
    /*
     * Open connection to
     * - unix socket
     * - or ssh then unix socket
     */
    fn open() -> Result<Self, VirshleError>;
    /*
     * Close connection
     */
    fn close() -> Result<(), VirshleError>;
    /*
     * Send an http GET request to socket.
     */
    fn get() -> Result<(), VirshleError>;
    /*
     * Send an http POST request to socket.
     */
    fn post() -> Result<(), VirshleError>;
    /*
     * Send an http PUT request to socket.
     */
    fn put() -> Result<(), VirshleError>;
    /*
     * Send the http request.
     * Internally used by get(), post() and put() methods.
     */
    fn execute(&mut self) -> Result<(), VirshleError>;
}
