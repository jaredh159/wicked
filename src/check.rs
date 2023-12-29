use std::borrow::Cow;
// reqwest docs: https://docs.rs/reqwest/0.10.7/reqwest/
// async process: https://docs.rs/async-process/latest/async_process/struct.Command.html

// make a connection, update some `status` col to `checking`, `unreachable`, `unparseable`

// bobs list:
// -> take a string, return me an interator of H1, ImgSrc, Title, and Text
// -> download an image onto the filesystem
// -> take an image src and test it with machine learning

#[derive(Debug, PartialEq)]
enum Content<'a> {
  Title(Cow<'a, str>),
  H1(Cow<'a, str>),
  ImgSrc(Cow<'a, str>),
  Text(Cow<'a, str>), // or vec?
}

fn html_content<'a>(dom: &'a tl::VDom<'a>) -> Vec<Content<'a>> {
  let mut content = Vec::new();
  let parser = dom.parser();
  for node in dom.nodes() {
    match node {
      tl::Node::Tag(tag) => match tag.name().as_utf8_str().as_ref() {
        "title" => {
          let title = tag.inner_text(parser);
          content.push(Content::Title(title));
        }
        "h1" => {
          let h1 = tag.inner_text(parser);
          content.push(Content::H1(h1));
        }
        "img" => {
          if let Some(src) = tag.attributes().get("src").flatten() {
            content.push(Content::ImgSrc(src.as_utf8_str()));
          }
        }
        tagname => {
          let children = tag.children();
          // let len = children.all(parser).len();
          let len = children.top().len();
          if len == 1 {
            let text = tag.inner_text(parser);
            content.push(Content::Text(text));
          } else {
            // println!("tag: {} has {} children", tagname, len);
            // if tagname == "div" {
            // dbg!(children.top());
            // for child in children.all() {
            //   dbg!(child);
            // }
            // }
          }
        }
      },
      tl::Node::Raw(raw) => {
        let text = raw.as_utf8_str();
        println!("raw: `{}`", text);
        // content.push(Content::Text(text));
      }
      tl::Node::Comment(_) => {}
    }
  }
  content
}

#[test]
fn test_html_content() {
  let input = r#"
  <html>
    <head>
      <title>My Title</title>
    </head>
    <body>
      <h1>My Header</h1>
      <div>
        <p>My Text</p>
        Rando text
      </div>
      <img src="https://example.com/image.png" />
    </body>
  "#;
  // fn html_content(dom: &tl::VDom) -> Vec<Content> {
  let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
  let content = html_content(&dom);
  assert_eq!(content.len(), 4);
  assert_eq!(content[0], Content::Title("My Title".into()));
  assert_eq!(content[1], Content::H1("My Header".into()));
  assert_eq!(content[2], Content::Text("My Text".into()));
  // assert_eq!(content[3], Content::Text("Rando text".into()));
  assert_eq!(content.len(), 5);
  assert_eq!(
    content[3],
    Content::ImgSrc("https://example.com/image.png".into())
  );
}

pub async fn domain(domain: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("https://{}", domain);
  // todo: don't follow redirects...
  // todo: timeout
  let response = reqwest::get(&url).await?;
  if !response.status().is_success() {
    todo!("handle bad res");
  }
  let body = response.text().await?;
  let dom = tl::parse(&body, tl::ParserOptions::default()).unwrap();
  let parser = dom.parser();
  // for node in dom.nodes() {
  //   match node {
  //     tl::Node::Tag(tag) => {
  //       let foo = tag.inner_text(parser);
  //       println!("tag: `<{}/>`", tag.name().as_utf8_str());
  //     }
  //     tl::Node::Raw(raw) => {}
  //     tl::Node::Comment(_) => {}
  //   }
  // }

  Ok(())
}
