#![allow(dead_code)]
#![allow(unused_variables)]

use tokio::sync::Mutex;

use std::sync::Arc;

mod bootstrap;
mod check;
mod db;
mod exec;
mod html;
mod stream;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
  dotenv::dotenv().ok();

  let cmd = std::env::args()
    .nth(1)
    .expect("missing required [cmd] arg (bootstrap|exec)");

  let client = db::connect().await?;

  match cmd.as_str() {
    "bootstrap" => bootstrap::run(&client).await?,
    "exec" => exec::run(Arc::new(Mutex::new(client))).await?,
    "check-domain" => {
      let domain = std::env::args()
        .next()
        .expect("missing required [domain] arg");
      let result = check::domain(&domain, &client).await;
      println!("\nresult: {result}");
    }
    _ => panic!("unknown command: `{cmd}`"),
  }
  Ok(())
}
