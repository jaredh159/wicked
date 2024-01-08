use crate::internal::*;

fn random_u32(max: u32) -> u32 {
  let mut rng = rand::thread_rng();
  rng.gen_range(0..max)
}

pub async fn random_unchecked_domain(
  client: Arc<Mutex<Client>>,
  total: u32,
) -> Result<String> {
  for _ in 0..500 {
    let random_number = random_u32(total);
    let query = format!("SELECT domain FROM domains WHERE id = {random_number}");

    let guard = client.lock().await;
    let row = guard.query_one(&query, &[]).await?;
    drop(guard);

    let domain: String = row.get(0);
    let query = format!("SELECT id FROM checked WHERE domain = '{domain}'");

    let guard = client.lock().await;
    let rows = guard.query(&query, &[]).await?;
    drop(guard);

    if rows.is_empty() {
      return Ok(domain); // haven't already checked
    }
  }
  Err("Failed to find unchecked domain in 500 attempts, likely none left to check".into())
}

pub async fn connect() -> Result<Client> {
  let url = std::env::var("DATABASE_URL").unwrap();
  let (client, connection) = tokio_postgres::connect(url.as_ref(), NoTls).await?;
  tokio::spawn(async move {
    if let Err(e) = connection.await {
      eprintln!("connection error: {e}");
    }
  });
  Ok(client)
}
