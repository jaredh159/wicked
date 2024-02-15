use crate::internal::*;

pub fn until_completed(shared: shared::Data) -> impl Stream<Item = ()> {
  UntilCompleteStream { iter: UntilComplete { shared } }
}

struct UntilComplete {
  shared: shared::Data,
}

impl Iterator for UntilComplete {
  type Item = ();

  fn next(&mut self) -> Option<Self::Item> {
    if self.shared.completed() {
      log::info!("finish until_completed stream");
      return None;
    }
    Some(())
  }
}

struct UntilCompleteStream {
  iter: UntilComplete,
}

impl Stream for UntilCompleteStream {
  type Item = ();

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    Poll::Ready(self.iter.next())
  }
}
