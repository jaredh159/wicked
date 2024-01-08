use crate::internal::*;

pub async fn run(db: &DbClient) -> Result<()> {
  println!("Bootstrapping database...");
  db.execute(DROP_DOMAINS_TABLE_SQL, &[]).await?;
  db.execute(CREATE_DOMAINS_TABLE_SQL, &[]).await?;
  db.execute(DROP_CHECKED_TABLE_SQL, &[]).await?;
  db.execute(CREATE_CHECKED_TABLE_SQL, &[]).await?;

  let fullsize_insert = chunk_stmt(CHUNK_SIZE, db).await?;
  let mut count = 0;
  let domain_iter = raw_domains_iter();
  for chunk in &domain_iter.into_iter().chunks(CHUNK_SIZE) {
    let params = chunk.collect::<Vec<String>>();
    if params.len() < CHUNK_SIZE {
      let remaining_insert = chunk_stmt(params.len(), db).await?;
      count += params.len();
      db.query_raw(&remaining_insert, params).await?;
    } else {
      db.query_raw(&fullsize_insert, params).await?;
      count += CHUNK_SIZE;
    }
    eprintln!("-> inserted {} domains...", en_us_separated_num(count));
  }

  println!(
    "\n√ FINISHED: {} unique domains inserted\n",
    en_us_separated_num(count)
  );

  Ok(())
}

async fn chunk_stmt(size: usize, db: &DbClient) -> Result<Statement> {
  let mut sql = String::from("INSERT INTO domains (domain) VALUES ");
  for i in 1..=size {
    sql.push_str(&format!("(${i}), "));
  }
  sql.pop();
  sql.pop();
  db.prepare(&sql).await.map_err(Into::into)
}

fn raw_domains_iter() -> impl Iterator<Item = String> {
  let filepath = std::env::var("RAW_DOMAINS_FILEPATH")
    .expect("missing required env var: `RAW_DOMAINS_FILEPATH`");
  let file = File::open(filepath).unwrap();
  let lines = io::BufReader::new(file).lines();
  lines
    .into_iter()
    .skip(1) // first line is header
    .map(StdResult::unwrap)
    .map(|line| line.split_whitespace().take(1).collect::<String>())
    .map(|mut domain| {
      assert_eq!(domain.pop(), Some('.'));
      domain
    })
    .unique()
}

fn env_paths() -> (String, String) {
  let input_path = std::env::var("RAW_DOMAINS_FILEPATH")
    .expect("missing required env var: `RAW_DOMAINS_FILEPATH`");
  let output_path = std::env::var("UNIQUE_DOMAINS_FILEPATH")
    .expect("missing required env var: `UNIQUE_DOMAINS_FILEPATH`");
  (input_path, output_path)
}

fn en_us_separated_num(i: usize) -> String {
  let mut s = String::new();
  let i_str = i.to_string();
  let a = i_str.chars().rev().enumerate();
  for (idx, val) in a {
    if idx != 0 && idx % 3 == 0 {
      s.insert(0, ',');
    }
    s.insert(0, val);
  }
  s
}

static CHUNK_SIZE: usize = 25_000;

static DROP_DOMAINS_TABLE_SQL: &str = "
DROP TABLE IF EXISTS domains
";

static CREATE_DOMAINS_TABLE_SQL: &str = "
CREATE TABLE domains (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  domain VARCHAR(82)
)
";

static DROP_CHECKED_TABLE_SQL: &str = "
DROP TABLE IF EXISTS checked
";

static CREATE_CHECKED_TABLE_SQL: &str = "
CREATE TABLE checked (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  domain VARCHAR(82),
  status VARCHAR(10)
)
";
