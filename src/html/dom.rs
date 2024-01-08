use html5ever::tendril::TendrilSink;
use html5ever::{local_name, parse_document, Attribute, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

#[derive(Debug, PartialEq, Eq)]
pub enum Content {
  Title(String),
  H1(String),
  ImgSrc(String),
  Text(String),
}

pub fn content(html: &str) -> Vec<Content> {
  let dom = parse_document(RcDom::default(), ParseOpts::default())
    .from_utf8()
    .read_from(&mut html.as_bytes())
    .unwrap();

  let mut content = Vec::new();
  walk(&dom.document, &mut content);
  content
}

fn walk(node: &Handle, content: &mut Vec<Content>) {
  match node.data {
    NodeData::Element { ref name, ref attrs, .. } => match name.local {
      local_name!("title") => {
        let mut title = String::new();
        for child in node.children.borrow().iter() {
          if let NodeData::Text { ref contents } = child.data {
            let text = contents.borrow();
            title.push_str(&text);
          }
        }
        content.push(Content::Title(title));
        return;
      }
      local_name!("h1") => {
        let mut h1_content = Vec::new();
        for child in node.children.borrow().iter() {
          walk(child, &mut h1_content);
        }
        let h1 = h1_content
          .iter()
          .filter_map(|c| match c {
            Content::Text(t) => Some(t.clone()),
            _ => None,
          })
          .collect::<Vec<_>>()
          .join(" ");
        content.push(Content::H1(h1));
        return;
      }
      local_name!("style") | local_name!("script") | local_name!("noscript") => {
        return;
      }
      local_name!("img") => {
        if let Some(Attribute { value, .. }) = attrs
          .borrow()
          .iter()
          .find(|a| a.name.local == local_name!("src"))
        {
          content.push(Content::ImgSrc(value.escape_default().to_string()));
        }
        return;
      }
      _ => {}
    },
    NodeData::Text { ref contents } => {
      let text = contents.borrow();
      if !text.trim().is_empty() {
        content.push(Content::Text(text.trim().escape_default().to_string()));
      }
    }
    _ => {}
  }
  for child in node.children.borrow().iter() {
    walk(child, content);
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
    let content = content(input);
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
