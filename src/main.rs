#![allow(dead_code)]
#![allow(unused_variables)]

mod bootstrap;
mod check;
mod db;

type Error = Box<dyn std::error::Error + Send + Sync>;

// 1. take file, bootstrap into another file
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
    .expect("missing required [cmd] arg (bootstrap|exec)");

  let client = db::connect().await?;

  match cmd.as_str() {
    "bootstrap" => bootstrap::run(&client).await?,
    _ => panic!("unknown command: `{}`", cmd),
  }
  Ok(())
}
