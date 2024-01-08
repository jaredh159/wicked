pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
pub type WordSpec = Vec<(String, Regex, u32)>;

pub use std::convert::Into;
pub use std::fmt;
pub use std::fs::File;
pub use std::io::{self, BufRead};
pub use std::pin::Pin;
pub use std::result::Result as StdResult;
pub use std::sync::atomic::{AtomicU32, Ordering};
pub use std::sync::Arc;
pub use std::task::{Context, Poll};
pub use std::time::Duration;

pub use futures::{Stream, StreamExt};
pub use itertools::Itertools;
pub use rand::Rng;
pub use regex::Regex;
pub use reqwest::Client as HttpClient;
pub use tokio::sync::Mutex;
pub use tokio_postgres::{Client as DbClient, NoTls, Statement};

pub mod db {
  pub use crate::db::*;
}
pub mod stream {
  pub use crate::stream::*;
}
pub mod html {
  pub use crate::html::*;
}
pub mod utils {
  pub use crate::utils::*;
}

#[derive(Debug)]
pub struct WickedError {
  pub message: String,
}

impl WickedError {
  pub fn new(message: &str) -> Self {
    Self { message: message.to_string() }
  }
}

impl fmt::Display for WickedError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WickedError: {}", self.message)
  }
}

impl std::error::Error for WickedError {}

pub fn e(message: impl Into<String>) -> Error {
  Box::new(WickedError { message: message.into() })
}
