use crate::internal::*;

pub fn until_completed(
  required: u32,
  reached: &AtomicU32,
) -> impl Stream<Item = ()> + '_ {
  UntilCompleteStream {
    iter: UntilComplete { reached, required },
  }
}

struct UntilComplete<'a> {
  reached: &'a AtomicU32,
  required: u32,
}

impl<'a> Iterator for UntilComplete<'a> {
  type Item = ();

  fn next(&mut self) -> Option<Self::Item> {
    if self.reached.load(Ordering::Relaxed) >= self.required {
      log::info!("finish until_completed stream");
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

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    Poll::Ready(self.iter.next())
  }
}
