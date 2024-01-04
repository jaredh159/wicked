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

pub use futures::{Stream, StreamExt};
pub use itertools::Itertools;
pub use rand::Rng;
pub use tl::{self, Node, ParserOptions, VDom};
pub use tokio::sync::Mutex;
pub use tokio_postgres::{Client, NoTls, Statement};

pub mod db {
  pub use crate::db::*;
}
pub mod stream {
  pub use crate::stream::*;
}
pub mod html {
  pub use crate::html::*;
}
