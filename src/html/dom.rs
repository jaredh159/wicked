use crate::internal::*;
//   nodes_to_skip += 1;

#[derive(Debug, PartialEq, Eq)]
pub enum Content<'a> {
  Title(Cow<'a, str>),
  H1(Cow<'a, str>),
  ImgSrc(Cow<'a, str>),
  Text(Cow<'a, str>),
}

pub fn content<'a>(dom: &'a VDom<'a>) -> impl Iterator<Item = Content<'a>> + 'a {
  ContentIter { dom, nodes: dom.nodes() }
}

struct ContentIter<'a> {
  dom: &'a VDom<'a>,
  nodes: &'a [Node<'a>],
}

impl<'a> ContentIter<'a> {
  fn remove_first(&mut self) {
    if !self.nodes.is_empty() {
      self.nodes = &self.nodes[1..];
    }
  }
}

impl<'a> Iterator for ContentIter<'a> {
  type Item = Content<'a>;
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let node = self.nodes.first()?;
      self.nodes = &self.nodes[1..];
      match node {
        Node::Tag(tag) => match tag.name().as_utf8_str().as_ref() {
          "title" => {
            let title = tag.inner_text(self.dom.parser());
            self.remove_first();
            return Some(Content::Title(title));
          }
          "h1" => {
            let h1 = tag.inner_text(self.dom.parser());
            let inner_html = tag.inner_html(self.dom.parser());
            let nodes_to_skip = tl::parse(&inner_html, ParserOptions::default())
              .map_or(1, |dom| dom.nodes().len());
            for _ in 0..nodes_to_skip {
              self.remove_first();
            }
            return Some(Content::H1(h1));
          }
          "img" => {
            if let Some(src) = tag.attributes().get("src").flatten() {
              return Some(Content::ImgSrc(src.as_utf8_str()));
            }
          }
          "style" | "script" => self.remove_first(),
          tagname => {
            if tag.children().top().len() == 1 {
              // let text = tag.inner_text(self.dom.parser());
              // self.remove_first();
              // return Some(Content::Text(text));
            }
          }
        },
        Node::Raw(raw) => {
          let text = raw.as_utf8_str();
          let trimmed = text.trim();
          if !trimmed.is_empty() {
            return Some(Content::Text(trimmed.to_string().into()));
          }
        }
        Node::Comment(_) => {}
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_html_content() {
    let input = r#"
      <html>
        <head>
          <title>My Title</title>
          <style>.my-class { color: red; }</style>
        </head>
        <body>
          <h1>My H1 Header</h1>
          <h1><b><em>Second h1</em></b></h1>
          <h1><b><em>Third h1</em></b> is odd</h1>
          <div>
            <p><em><b><code>My P Text</code></b></em></p>
            <!-- comment -->
            Rando text
            <span>In Span</span>
          </div>
          <img src="https://example.com/image.png" />
        </body>
      "#;
    let dom = tl::parse(input, ParserOptions::default()).unwrap();
    let content = content(&dom).collect::<Vec<_>>();
    // eprintln!();
    assert_eq!(
      content,
      vec![
        Content::Title("My Title".into()),
        Content::H1("My H1 Header".into()),
        Content::H1("Second h1".into()),
        Content::H1("Third h1 is odd".into()),
        Content::Text("My P Text".into()),
        Content::Text("Rando text".into()),
        Content::Text("In Span".into()),
        Content::ImgSrc("https://example.com/image.png".into()),
      ]
    );
  }
}
