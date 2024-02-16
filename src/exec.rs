use crate::internal::*;

// parallelism higher than 10 seems to cause DNS rate limiting...
const PARALLELISM: u32 = 8;

pub async fn run(shared_db: Arc<Mutex<DbClient>>, conf: &Config) -> Result<()> {
  let sample_size = 1000;
  log::info!(
    "starting exec::run(), sample_size: {}, parallelism: {}",
    sample_size,
    PARALLELISM
  );
  let data = shared::Data::new(
    167_300_740,
    sample_size, // todo: pass
    conf.clone(),
    Arc::clone(&shared_db),
  );

  // let db = Arc::clone(&shared_db);
  // let db = db.lock().await;
  // let row = db.query_one(QUERY_COUNT_COMPLETE, &[]).await?;
  // drop(db);

  // let db_count: u32 = row.get::<_, String>(0).parse().unwrap();
  // data.attempted.store(db_count, Ordering::Relaxed);

  let tasks = stream::until_completed(data.clone())
    .map(|()| process_domain(data.clone()))
    .buffer_unordered(u32::min(PARALLELISM, data.sample_size) as usize);

  tasks.collect::<Vec<_>>().await;
  let num_attempted = data.attempted.load(Ordering::Acquire);
  let num_found_porn = data.porn.load(Ordering::Acquire);
  let num_reached = data.reached.load(Ordering::Acquire);
  log::info!(
    "finished exec::run(), porn: {} ({}%), attempted: {}, reached: {}, {}% unreachable",
    num_found_porn,
    utils::percent_str(num_found_porn, num_reached),
    num_attempted,
    num_reached,
    utils::percent_str(num_attempted - num_reached, num_attempted),
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

fn process_domain(data: shared::Data) -> impl Future<Output = Result<()>> {
  let conf = data.config.clone();
  async move {
    let (domain, id) = db::random_unchecked_domain(data.db.clone(), data.total).await?;
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

    if !data.aborted.load(Ordering::Relaxed) {
      db::insert_result(data.db.clone(), id, &domain, result).await?;
    }
    Ok(())
  }
}

static SELECT_RANDOM: &str = "SELECT domain FROM domains WHERE id =";
static SELECT_CHECKED: &str = "SELECT id FROM checked WHERE domain =";
static QUERY_COUNT_COMPLETE: &str =
  "SELECT COUNT(*)::text FROM checked WHERE status != 'unreachable'";
