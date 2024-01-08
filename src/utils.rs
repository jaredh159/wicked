use regex::RegexBuilder;

use crate::internal::*;

pub fn to_spec(input: Vec<(String, u32)>) -> WordSpec {
  input
    .into_iter()
    .map(|(s, n)| {
      let pattern = format!(r"\b{}\b", s);
      (
        s,
        RegexBuilder::new(&pattern)
          .case_insensitive(true)
          .build()
          .unwrap_or_else(|_| panic!("unable to create regex from word: `{}`", pattern)),
        n,
      )
    })
    .collect()
}
