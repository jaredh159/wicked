mod bootstrap;
mod check;
mod db;
mod exec;
mod html;
mod internal;
mod stream;

use internal::*;

#[tokio::main]
async fn main() -> Result<()> {
  dotenv::dotenv().ok();

  let mut args = std::env::args().skip(1);
  let cmd = args
    .next()
    .expect("missing required [cmd] arg (bootstrap|exec|check-domain)");

  let client = db::connect().await?;

  match cmd.as_str() {
    "bootstrap" => bootstrap::run(&client).await?,
    "exec" => exec::run(Arc::new(Mutex::new(client))).await?,
    "check-domain" => {
      let domain = args.next().expect("missing required [domain] arg");
      let result = check::domain(&domain, &client).await?;
      println!("\nresult: {result}");
    }
    _ => panic!("unknown command: `{cmd}`"),
  }
  Ok(())
}
