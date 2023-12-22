use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  for domain in domains() {
    println!("domain is: {}", domain);
  }
  Ok(())
}

fn domains() -> impl Iterator<Item = String> {
  let file = File::open("/Users/jared/Desktop/com.txt").unwrap();
  let lines = io::BufReader::new(file).lines();
  lines
    .into_iter()
    .skip(1)
    .take(5)
    .map(|result| result.unwrap())
    .map(|line| line.split_whitespace().take(1).collect::<String>())
}
