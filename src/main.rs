mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenv::dotenv().ok();
  let cmd = std::env::args().nth(1);

  if cmd == Some(String::from("db:reset")) {
    let client = db::connect().await?;
    db::reset(&client, db::Conf { limit: 50_000, offset: 200_000_000 }).await?;
  }

  Ok(())
}
