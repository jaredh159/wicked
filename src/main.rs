#![allow(dead_code)]
#![allow(unused_variables)]

mod check;
mod db;
mod dedupe;

type Error = Box<dyn std::error::Error + Send + Sync>;

// 1. take file, dedupe into another file
// 2. rework db reset to drop table, recreate
// 3. add run --sample=100_000, w/ plan:
//      - spawn tasks to:
//      - take a random domain from file
//      - if it's already been processed, go back a step
//      - test for reachability

#[tokio::main]
async fn main() -> Result<(), Error> {
  dotenv::dotenv().ok();
  let cmd = std::env::args()
    .nth(1)
    .expect("missing required [cmd] arg (dedupe|db-reset)");
  let client = db::connect().await?;

  match cmd.as_str() {
    "dedupe" => dedupe::run()?,
    "db-reset" => db::reset(&client, db::Conf { limit: 0, offset: 0 }).await?,
    _ => panic!("unknown command: `{}`", cmd),
  }
  Ok(())
}
