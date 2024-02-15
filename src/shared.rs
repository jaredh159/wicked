use crate::internal::*;

#[derive(Clone)]
pub struct Data {
  pub attempted: &'static AtomicU32,
  pub reached: &'static AtomicU32,
  pub porn: &'static AtomicU32,
  pub sites: Arc<Mutex<Vec<&'static str>>>,
  pub aborted: &'static AtomicBool,
  pub db: Arc<Mutex<DbClient>>,
  pub total: u32,
  pub sample_size: u32,
  pub config: Config,
}

impl Data {
  pub fn completed(&self) -> bool {
    let reached = self.reached.load(Ordering::Relaxed);
    if reached >= self.sample_size {
      return true;
    }
    let num_attempted = self.attempted.load(Ordering::Relaxed);
    let percent_unreachable = percent(num_attempted - reached, num_attempted);
    let dns_limiting_detected = match num_attempted {
      ..=100 => false,
      101..=200 => percent_unreachable >= 79,
      201..=300 => percent_unreachable >= 76,
      301..=400 => percent_unreachable >= 74,
      401..=500 => percent_unreachable >= 73,
      501..=600 => percent_unreachable >= 72,
      601..=700 => percent_unreachable >= 71,
      _ => percent_unreachable >= 70,
    };
    if dns_limiting_detected {
      log::error!("suspected DNS rate limiting detected, aborting");
    }
    dns_limiting_detected
  }

  pub async fn log_progress(&self) {
    let porn = self.porn.load(Ordering::Acquire);
    let num_reached = self.reached.load(Ordering::Acquire);
    let num_attempted = self.attempted.load(Ordering::Acquire);
    if num_attempted >= 20 && num_attempted % 20 == 0 {
      log::info!(
        "progress: {}/{} ({}%) finished, {}/{} ({}%) porn, {}/{} ({}%) unreachable",
        num_reached,
        self.sample_size,
        percent(num_reached, self.sample_size),
        porn,
        num_reached,
        percent(porn, num_reached),
        num_attempted - num_reached,
        num_attempted,
        percent(num_attempted - num_reached, num_attempted),
      );
    } else {
      log::debug!(
        "progress: {}/{} ({}%) finished, {}/{} ({}%) porn, {}/{} ({}%) unreachable",
        num_reached,
        self.sample_size,
        percent(num_reached, self.sample_size),
        porn,
        num_reached,
        percent(porn, num_reached),
        num_attempted - num_reached,
        num_attempted,
        percent(num_attempted - num_reached, num_attempted),
      );
    }
    if num_attempted >= 100 && num_attempted % 100 == 0 {
      let guard = self.sites.lock().await;
      if !guard.is_empty() {
        log::info!("porn sites:");
        for site in &*guard {
          log::info!(" -> {}", site);
        }
      }
    }
  }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
fn percent(part: u32, total: u32) -> i32 {
  if total == 0 {
    0
  } else {
    ((part as f32 / total as f32) * 100.0) as i32
  }
}
