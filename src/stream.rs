use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Context, Poll};

use futures::Stream;

pub fn until_completed(required: u32, completed: &AtomicU32) -> impl Stream<Item = ()> + '_ {
  UntilCompleteStream {
    iter: UntilComplete { completed, required },
  }
}

struct UntilComplete<'a> {
  completed: &'a AtomicU32,
  required: u32,
}

impl<'a> Iterator for UntilComplete<'a> {
  type Item = ();
  fn next(&mut self) -> Option<Self::Item> {
    if self.completed.load(Ordering::Relaxed) >= self.required {
      return None;
    }
    Some(())
  }
}

struct UntilCompleteStream<'a> {
  iter: UntilComplete<'a>,
}

impl Stream for UntilCompleteStream<'_> {
  type Item = ();
  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    Poll::Ready(self.iter.next())
  }
}
