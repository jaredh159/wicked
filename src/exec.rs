use crate::internal::*;

// parallelism higher than 10 seems to cause DNS rate limiting...
const PARALLELISM: u32 = 10;

pub async fn run(shared_db: Arc<Mutex<DbClient>>, conf: &Config) -> Result<()> {
  static ATTEMPTED: AtomicU32 = AtomicU32::new(0);
  static REACHED: AtomicU32 = AtomicU32::new(0);
  static PORN: AtomicU32 = AtomicU32::new(0);
  let porn_sites = Arc::new(Mutex::new(Vec::<String>::new()));
  log::info!("starting exec::run()");
  let total = 167_300_740; // todo: pass
  let sample_size: u32 = 1000; // todo: pass

  let db = Arc::clone(&shared_db);
  let db = db.lock().await;
  let row = db.query_one(QUERY_COUNT_COMPLETE, &[]).await?;
  drop(db);

  let db_count: u32 = row.get::<_, String>(0).parse().unwrap();
  ATTEMPTED.store(db_count, Ordering::Relaxed);

  let tasks = stream::until_completed(sample_size, &REACHED)
    .map(|()| {
      try_check_domain(
        &ATTEMPTED,
        &REACHED,
        &PORN,
        total,
        sample_size,
        Arc::clone(&shared_db),
        Arc::clone(&porn_sites),
        conf,
      )
    })
    .buffer_unordered(u32::min(PARALLELISM, sample_size) as usize);

  tasks.collect::<Vec<_>>().await;
  let final_attempted = ATTEMPTED.load(Ordering::Acquire);
  let found_porn = PORN.load(Ordering::Acquire);
  let final_reached = REACHED.load(Ordering::Acquire);
  log::info!(
    "finished exec::run(), porn: {} ({}%), attempted: {}, reached: {}, {}% unreachable",
    found_porn,
    percent(found_porn, final_reached),
    final_attempted,
    final_reached,
    percent(final_attempted - final_reached, final_attempted),
  );

  let guard = porn_sites.lock().await;
  if !guard.is_empty() {
    println!("porn sites:");
    for site in &*guard {
      println!(" -> {}", site);
    }
  }
  Ok(())
}

fn try_check_domain(
  attempted: &'static AtomicU32,
  reached: &'static AtomicU32,
  found_porn: &'static AtomicU32,
  total: u32,
  sample_size: u32,
  shared_db: Arc<Mutex<DbClient>>,
  porn_sites: Arc<Mutex<Vec<String>>>,
  conf: &Config,
) -> impl Future<Output = Result<()>> {
  let conf = conf.clone();
  async move {
    let domain = db::random_unchecked_domain(shared_db.clone(), total).await?;
    log::debug!("start checking domain: {domain}",);
    let result = check::domain(&domain, &conf).await;
    let updated_attempted = attempted.fetch_add(1, Ordering::Relaxed);

    if let DomainResult::Tested(result) = &result {
      reached.fetch_add(1, Ordering::Relaxed);
      if result.is_porn {
        found_porn.fetch_add(1, Ordering::Relaxed);
        let mut guard = porn_sites.lock().await;
        guard.push(domain.clone());
      }
    }

    log::info!("finish checking domain: {domain}, result: {:?}", result);
    let porn = found_porn.load(Ordering::Acquire);
    let num_reached = reached.load(Ordering::Acquire);
    log::debug!(
      "progress: {}/{} ({}%) finished, {}/{} ({}%) porn, {}/{} ({}%) unreachable",
      num_reached,
      sample_size,
      percent(num_reached, sample_size),
      porn,
      num_reached,
      percent(porn, num_reached),
      updated_attempted - num_reached,
      updated_attempted,
      percent(updated_attempted - num_reached, updated_attempted),
    );
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
