use crate::internal::*;

pub fn check(content: &[html::Content], conf: &Config) -> usize {
  content.iter().fold(0, |total, c| {
    total
      + match c {
        html::Content::Title(s) => weighted(s, &conf.words, conf.title_tag_multiplier),
        html::Content::H1(s) => weighted(s, &conf.words, conf.h1_tag_multiplier),
        html::Content::Text(s) => weighted(s, &conf.words, conf.other_text_multiplier),
        html::Content::ImgSrc(_) => 0,
      }
  })
}

fn weighted(text: &str, words: &[(String, Regex, usize)], weight: usize) -> usize {
  words.iter().fold(0, |total, (word, regex, points)| {
    let num_matches = regex.captures_iter(text).count();
    total + num_matches * points * weight
  })
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::html::Content::*;

  #[test]
  fn test_check_words() {
    let cases: Vec<(Vec<html::Content>, Vec<(String, usize)>, usize)> = vec![
      (
        vec![
          Title("foo goat frog".to_string()), // 3 * 5, 2 * 5 = 25
          H1("goat baz innergoatnope".to_string()), // 3 * 2, 0 * 2 = 6
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
          Text("frog such frog goat".to_string()),
          ImgSrc("goat.jpg".to_string()),
          ImgSrc("frog.jpg".to_string()),
        ],
        vec![(String::from("goat"), 3), (String::from("frog"), 2)],
        38,
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
      // let actual = check(&content, &spec::map_regex(spec));
      let actual = check(
        &content,
        &Config {
          title_tag_multiplier: 5,
          h1_tag_multiplier: 2,
          other_text_multiplier: 1,
          words: spec::map_regex(spec),
        },
      );
      assert_eq!(actual, expected);
    }
  }
}
