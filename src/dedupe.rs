use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};

use itertools::Itertools;

use super::Error;

pub fn run() -> Result<(), Error> {
  let input_path = std::env::var("RAW_DOMAINS_FILEPATH")
    .expect("missing required env var: `RAW_DOMAINS_FILEPATH`");
  let output_path = std::env::var("UNIQUE_DOMAINS_FILEPATH")
    .expect("missing required env var: `UNIQUE_DOMAINS_FILEPATH`");

  let file = File::create(&output_path).unwrap();
  let mut writer = BufWriter::new(file);
  let mut count = 0;
  raw_domains_iter(&input_path).for_each(|domain| {
    if count % 100_000 == 0 {
      eprintln!(
        " -> processed {} unique domains...",
        en_us_separated_num(count)
      );
    }
    writer.write_all(domain.as_bytes()).unwrap();
    writer.write_all(b"\n").unwrap();
    count += 1;
  });

  writer.flush().unwrap();

  println!(
    "\n√ FINISHED: {} unique domains stored in {}\n",
    count, output_path
  );

  Ok(())
}

fn raw_domains_iter(filepath: &str) -> impl Iterator<Item = String> {
  let file = File::open(filepath).unwrap();
  let lines = io::BufReader::new(file).lines();
  lines
    .into_iter()
    .skip(1) // first line is header
    .map(|result| result.unwrap())
    .map(|line| line.split_whitespace().take(1).collect::<String>())
    .map(|mut domain| {
      assert_eq!(domain.pop(), Some('.'));
      domain
    })
    .unique()
}

fn en_us_separated_num(i: i32) -> String {
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
