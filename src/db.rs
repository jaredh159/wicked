use crate::internal::*;

fn random_u32(max: u32) -> u32 {
  let mut rng = rand::thread_rng();
  rng.gen_range(0..max)
}

pub async fn random_unchecked_domain(
  client: Arc<Mutex<DbClient>>,
  total: u32,
) -> Result<(String, u32)> {
  for _ in 0..500 {
    let id = random_u32(total);
    let query = format!("SELECT domain FROM domains WHERE id = {id}");
    let row = client.lock().await.query_one(&query, &[]).await.unwrap();

    let domain: String = row.get(0);
    let query = format!("SELECT id FROM checked WHERE id = '{id}'");
    let rows = client.lock().await.query(&query, &[]).await.unwrap();

    if rows.is_empty() {
      return Ok((domain, id)); // haven't already checked
    }

    log::error!("domain already checked: {domain}, retrying...");
  }
  Err("Failed to find unchecked domain in 500 attempts, likely none left to check".into())
}

pub async fn insert_result(
  client: Arc<Mutex<DbClient>>,
  id: u32,
  domain: &str,
  result: DomainResult,
) -> Result<()> {
  let query = format!(
    r#"
      INSERT INTO checked (id, domain, status) VALUES
      ({id}, '{domain}', '{}')
    "#,
    match result {
      DomainResult::Unreachable => "unreach",
      DomainResult::Parked => "parked",
      DomainResult::Tested(TestResult { is_porn: true, .. }) => "porn",
      DomainResult::Tested(TestResult { is_porn: false, .. }) => "notporn",
    }
  );
  log::trace!("starting insert result for domain: {domain}");
  client.lock().await.execute(&query, &[]).await.unwrap();
  log::trace!("finished insert result for domain: {domain}");
  Ok(())
}

pub async fn connect() -> Result<DbClient> {
  let url = std::env::var("DATABASE_URL").unwrap();
  let (client, connection) = tokio_postgres::connect(url.as_ref(), NoTls).await?;
  tokio::spawn(async move {
    if let Err(error) = connection.await {
      log::error!("postgres connection error: {error}");
    }
  });
  Ok(client)
}
