use std::fs::File;
use std::io::{self, BufRead};

use itertools::Itertools;
use tokio_postgres as pg;

pub async fn connect() -> Result<pg::Client, pg::Error> {
  let url = std::env::var("DATABASE_URL").unwrap();
  let (client, connection) = pg::connect(url.as_ref(), pg::NoTls).await?;

  tokio::spawn(async move {
    if let Err(e) = connection.await {
      eprintln!("connection error: {}", e);
    }
  });

  Ok(client)
}

pub struct Conf {
  pub limit: usize,
  pub offset: usize,
}

static CHUNK_SIZE: usize = 25_000;
static TOTAL_RECORDS: usize = 414_865_650;

pub async fn random_domain(client: &pg::Client) -> Result<String, pg::Error> {
  let query = "SELECT domain FROM domains TABLESAMPLE SYSTEM (1) LIMIT 1";
  let row = client.query_one(query, &[]).await?;
  let domain: String = row.get(0);
  if CHUNK_SIZE > 0 {
    return Ok("netrivet.com".to_string()); // temp
  }
  Ok(domain)
}

pub async fn reset(client: &pg::Client, config: Conf) -> Result<(), pg::Error> {
  println!("Resetting database...");
  client.execute("DELETE FROM domains", &[]).await?;

  let num_chunks = config.limit / CHUNK_SIZE;
  let fullsize_insert = chunk_stmt(CHUNK_SIZE, client).await?;

  let domain_iter = domains(&config);
  let mut count = 1;
  for chunk in &domain_iter.into_iter().chunks(CHUNK_SIZE) {
    eprintln!(
      "  {}/{}, chunk size: {}, total: {}",
      count,
      num_chunks,
      CHUNK_SIZE,
      usize::min(config.limit, TOTAL_RECORDS)
    );
    let params = chunk.collect::<Vec<String>>();
    if params.len() < CHUNK_SIZE {
      let remaining_insert = chunk_stmt(params.len(), client).await?;
      client.query_raw(&remaining_insert, params).await?;
    } else {
      client.query_raw(&fullsize_insert, params).await?;
    }
    count += 1;
  }
  Ok(())
}

async fn chunk_stmt(size: usize, client: &pg::Client) -> Result<pg::Statement, pg::Error> {
  let mut sql = String::from("INSERT INTO domains (domain) VALUES ");
  for i in 1..=size {
    sql.push_str(&format!("(${}), ", i));
  }
  sql.pop();
  sql.pop();
  client.prepare(&sql).await
}

fn domains(config: &Conf) -> impl Iterator<Item = String> {
  let path = std::env::var("DOMAINS_FILE_PATH").unwrap();
  let file = File::open(path).unwrap();
  let lines = io::BufReader::new(file).lines();
  lines
    .into_iter()
    .skip(1 + config.offset) // first line is header
    .map(|result| result.unwrap())
    .map(|line| line.split_whitespace().take(1).collect::<String>())
    .map(|mut domain| {
      assert_eq!(domain.pop(), Some('.'));
      domain
    })
    .unique()
    .take(config.limit)
}
