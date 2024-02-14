pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

pub use std::borrow::Cow;
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

pub use env_logger::Env;
pub use futures::{Future, Stream, StreamExt};
pub use itertools::Itertools;
pub use log;
pub use rand::Rng;
pub use regex::Regex;
pub use reqwest::Client as HttpClient;
pub use serde::Deserialize;
pub use tokio::sync::Mutex;
pub use tokio_postgres::{Client as DbClient, NoTls, Statement};
pub use uuid::Uuid;

pub use crate::check::{DomainResult, TestResult};
pub use crate::config::Config;

pub mod db {
  pub use crate::db::*;
}
pub mod stream {
  pub use crate::stream::*;
}
pub mod html {
  pub use crate::html::*;
}

pub mod check {
  pub use crate::check::*;
}

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
    write!(f, "{}", self.message)
  }
}

impl fmt::Debug for WickedError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self)
  }
}

impl std::error::Error for WickedError {}

pub fn e(message: impl Into<String>) -> Error {
  Box::new(WickedError { message: message.into() })
}
