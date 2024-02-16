use crate::internal::*;

pub async fn run(shared_db: Arc<Mutex<DbClient>>, conf: &Config) -> Result<()> {
  log::info!(
    "starting exec::run(), sample_size: {}, parallelism: {}",
    conf.sample_size,
    conf.parallelism
  );

  let data = shared::Data::new(
    match conf.total_num_domains {
      Some(num) => num,
      None => domain_count(shared_db.clone()).await,
    },
    conf.clone(),
    Arc::clone(&shared_db),
  );

  let futs = stream::until_completed(data.clone())
    .map(|()| process_domain(data.clone()))
    .buffer_unordered(conf.parallelism as usize);

  futs.collect::<Vec<_>>().await;
  log_complete(&data).await;
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

async fn domain_count(db: Arc<Mutex<DbClient>>) -> u32 {
  log::info!("start get db domain count, bypass w/ `total_num_domains` in config");
  let row = db
    .lock()
    .await
    .query_one(QUERY_COUNT_DOMAINS, &[])
    .await
    .unwrap();
  let count = row.get::<_, String>(0).parse().unwrap();
  log::info!("finish get db domain count: {count}");
  count
}

async fn log_complete(data: &shared::Data) {
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
    log::info!("porn sites:");
    for site in &*guard {
      log::info!(" -> {}", site);
    }
  }
}

static SELECT_RANDOM: &str = "SELECT domain FROM domains WHERE id =";
static SELECT_CHECKED: &str = "SELECT id FROM checked WHERE domain =";
static QUERY_COUNT_DOMAINS: &str = "SELECT COUNT(*)::text FROM domains";
static QUERY_COUNT_COMPLETE: &str =
  "SELECT COUNT(*)::text FROM checked WHERE status != 'unreachable'";
