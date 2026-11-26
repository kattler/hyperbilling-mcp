//! HyperBilling Open API communication.

pub mod client;
pub mod error;

pub use client::{Client, Credentials, Reply};
pub use error::Error;
