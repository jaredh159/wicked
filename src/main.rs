mod check;
mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  dotenv::dotenv().ok();
  let cmd = std::env::args().nth(1);
  let client = db::connect().await?;

  if cmd == Some(String::from("db:reset")) {
    db::reset(&client, db::Conf { limit: 50_000, offset: 250_000_000 }).await?;
  }

  let domain = db::random_domain(&client).await?;
  check::domain(domain).await?;

  Ok(())
}
