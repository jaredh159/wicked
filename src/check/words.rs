use crate::internal::*;

pub fn check(content: &[html::Content], spec: &WordSpec) -> u32 {
  content.iter().fold(0, |total, c| {
    total
      + match c {
        html::Content::Title(s) => check_weighted(s, spec, 5),
        html::Content::H1(s) => check_weighted(s, spec, 2),
        html::Content::Text(s) => check_weighted(s, spec, 1),
        html::Content::ImgSrc(_) => 0,
      }
  })
}

fn check_weighted(text: &str, spec: &WordSpec, weight: u32) -> u32 {
  let mut total = 0;
  for (word, regex, points) in spec {
    if regex.is_match(text) {
      total += points * weight;
    }
  }
  total
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::html::Content::*;

  #[test]
  fn test_check_words() {
    let cases: Vec<(Vec<html::Content>, Vec<(String, u32)>, u32)> = vec![
      (
        vec![
          Title("foo goat frog".to_string()), // 3 * 5, 2 * 5 = 25
          H1("goat baz innergoatnope".to_string()), // 3 * 2, 0 * 2 = 10
          Text("frog such goat".to_string()), // 3 * 1, 2 * 1 = 5
          ImgSrc("goat.jpg".to_string()),     // ignored
          ImgSrc("frog.jpg".to_string()),     // ignored
        ],
        vec![(String::from("goat"), 3), (String::from("frog"), 2)],
        36,
      ),
      (
        vec![
          Title("foo goat frog".to_string()),
          H1("goat baz innergoatnope".to_string()),
          Text("frog such goat".to_string()),
          ImgSrc("goat.jpg".to_string()),
          ImgSrc("frog.jpg".to_string()),
        ],
        vec![(String::from("GoAt"), 3), (String::from("fROg"), 2)],
        36,
      ),
      (
        vec![
          Title("foo goat frog".to_string()),
          H1("goat baz innergoatnope".to_string()),
          Text("frog such goat".to_string()),
          ImgSrc("goat.jpg".to_string()),
          ImgSrc("frog.jpg".to_string()),
        ],
        vec![(String::from("bob"), 3), (String::from("banana"), 2)],
        0,
      ),
    ];

    for (content, spec, expected) in cases {
      let actual = check(&content, &utils::to_spec(spec));
      assert_eq!(actual, expected);
    }
  }
}
