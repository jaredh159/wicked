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
  pub fn new(total: u32, config: Config, db: Arc<Mutex<DbClient>>) -> Self {
    static ATTEMPTED: AtomicU32 = AtomicU32::new(0);
    static REACHED: AtomicU32 = AtomicU32::new(0);
    static PORN: AtomicU32 = AtomicU32::new(0);
    static ABORTED: AtomicBool = AtomicBool::new(false);
    Self {
      attempted: &ATTEMPTED,
      reached: &REACHED,
      porn: &PORN,
      aborted: &ABORTED,
      sites: Arc::new(Mutex::new(Vec::new())),
      db,
      total,
      sample_size: config.sample_size,
      config,
    }
  }

  pub fn completed(&self) -> bool {
    let reached = self.reached.load(Ordering::Relaxed);
    if reached >= self.sample_size {
      return true;
    }
    let num_attempted = self.attempted.load(Ordering::Relaxed);
    let unreachable = num_attempted - reached;
    let percent_unreachable = utils::percent(unreachable, num_attempted);
    let dns_limiting_detected = match num_attempted {
      ..=100 => false,
      101..=200 => percent_unreachable >= 85.0,
      201..=300 => percent_unreachable >= 79.0,
      301..=400 => percent_unreachable >= 77.0,
      401..=500 => percent_unreachable >= 75.0,
      501..=600 => percent_unreachable >= 74.0,
      601..=700 => percent_unreachable >= 73.0,
      _ => percent_unreachable >= 72.0,
    };
    if dns_limiting_detected {
      self.aborted.store(true, Ordering::SeqCst);
      log::error!(
        "suspected DNS rate limiting detected, aborting: {}/{} ({})%",
        unreachable,
        num_attempted,
        utils::percent_str(unreachable, num_attempted)
      );
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
        utils::percent_str(num_reached, self.sample_size),
        porn,
        num_reached,
        utils::percent_str(porn, num_reached),
        num_attempted - num_reached,
        num_attempted,
        utils::percent_str(num_attempted - num_reached, num_attempted),
      );
    } else {
      log::debug!(
        "progress: {}/{} ({}%) finished, {}/{} ({}%) porn, {}/{} ({}%) unreachable",
        num_reached,
        self.sample_size,
        utils::percent_str(num_reached, self.sample_size),
        porn,
        num_reached,
        utils::percent_str(porn, num_reached),
        num_attempted - num_reached,
        num_attempted,
        utils::percent_str(num_attempted - num_reached, num_attempted),
      );
    }
    if num_attempted >= 200 && num_attempted % 200 == 0 {
      let guard = self.sites.lock().await;
      if !guard.is_empty() {
        log::info!("porn sites:");
        for site in &*guard {
          log::info!("    -> {}", site);
        }
      }
    }
  }
}
