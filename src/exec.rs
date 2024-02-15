use crate::internal::*;

// parallelism higher than 10 seems to cause DNS rate limiting...
const PARALLELISM: u32 = 10;

pub async fn run(shared_db: Arc<Mutex<DbClient>>, conf: &Config) -> Result<()> {
  static ATTEMPTED: AtomicU32 = AtomicU32::new(0);
  static REACHED: AtomicU32 = AtomicU32::new(0);
  static PORN: AtomicU32 = AtomicU32::new(0);
  static ABORTED: AtomicBool = AtomicBool::new(false);

  let data = shared::Data {
    attempted: &ATTEMPTED,
    reached: &REACHED,
    porn: &PORN,
    aborted: &ABORTED,
    sites: Arc::new(Mutex::new(Vec::new())),
    db: Arc::clone(&shared_db),
    total: 167_300_740, // todo: pass
    sample_size: 1000,  // todo: pass
    config: conf.clone(),
  };

  log::info!("starting exec::run()");

  let db = Arc::clone(&shared_db);
  let db = db.lock().await;
  let row = db.query_one(QUERY_COUNT_COMPLETE, &[]).await?;
  drop(db);

  let db_count: u32 = row.get::<_, String>(0).parse().unwrap();
  data.attempted.store(db_count, Ordering::Relaxed);

  let tasks = stream::until_completed(data.clone())
    .map(|()| try_check_domain(data.clone()))
    .buffer_unordered(u32::min(PARALLELISM, data.sample_size) as usize);

  tasks.collect::<Vec<_>>().await;
  let num_attempted = data.attempted.load(Ordering::Acquire);
  let num_found_porn = data.porn.load(Ordering::Acquire);
  let num_reached = data.reached.load(Ordering::Acquire);
  log::info!(
    "finished exec::run(), porn: {} ({}%), attempted: {}, reached: {}, {}% unreachable",
    num_found_porn,
    percent(num_found_porn, num_reached),
    num_attempted,
    num_reached,
    percent(num_attempted - num_reached, num_attempted),
  );

  let guard = data.sites.lock().await;
  if !guard.is_empty() {
    println!("porn sites:");
    for site in &*guard {
      println!(" -> {}", site);
    }
  }
  Ok(())
}

fn try_check_domain(data: shared::Data) -> impl Future<Output = Result<()>> {
  let conf = data.config.clone();
  async move {
    let domain = db::random_unchecked_domain(data.db.clone(), data.total).await?;
    log::debug!("start checking domain: {domain}",);
    let result = check::domain(&domain, &conf).await;
    data.attempted.fetch_add(1, Ordering::Relaxed);

    if let DomainResult::Tested(result) = &result {
      data.reached.fetch_add(1, Ordering::Relaxed);
      if result.is_porn {
        data.porn.fetch_add(1, Ordering::Relaxed);
        let mut guard = data.sites.lock().await;
        guard.push(domain.clone().leak());
      }
    }

    log::debug!("finish checking domain: {domain}, result: {:?}", result);
    data.log_progress().await;
    Ok(())
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

#[cfg(test)]
#[test]
fn test_percent() {
  assert_eq!(percent(0, 0), 0);
  assert_eq!(percent(0, 1), 0);
  assert_eq!(percent(1, 1), 100);
  assert_eq!(percent(1, 2), 50);
  assert_eq!(percent(2, 8), 25);
}

static SELECT_RANDOM: &str = "SELECT domain FROM domains WHERE id =";
static SELECT_CHECKED: &str = "SELECT id FROM checked WHERE domain =";
static QUERY_COUNT_COMPLETE: &str =
  "SELECT COUNT(*)::text FROM checked WHERE status != 'unreachable'";
