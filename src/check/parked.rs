pub fn check(domain: &str, html: &str) -> bool {
  if html.len() < 400
    && (html.contains("parking-lander")
      || html.contains(r#"<meta http-equiv="Refresh" content="0;url=defaultsite" />"#))
  {
    return true;
  }
  if html.len() > 6000 {
    let head = &html.chars().take(2000).collect::<String>();
    let search_strings = [
      "sedoparking.com",
      "<title>Checkdomain Parking",
      "Domain parking page",
      "/sedo.com",
      "comp-is-parked",
      "This domain is parked",
      "Buy with Epik.com</title>",
    ];
    for s in &search_strings {
      if head.contains(s) {
        return true;
      }
    }
    if !&html
      .chars()
      .take(300)
      .collect::<String>()
      .contains("<script>")
    {
      return false;
    }
  }
  let search_strings = [
    "sedoparking.com",
    "comp-is-parked",
    "class=\"ParkingPage",
    "Who owns the domain?",
    "Want your own domain name?",
    "Domain parking page",
    "This domain name is parked for FREE",
    "This domain is parked",
    "COMING SOON to APLUS.NET",
    "/parking-lander/",
    "/parked/[% parked_type %]/",
    "href=\"https://www.domainnameshop.com/whois\"",
    "free domain names, domain name, front page hosting, web site, web design, domain name registration",
  ];
  for s in &search_strings {
    if html.contains(s) {
      return true;
    }
  }

  let domain_strings = [
    "<title>Welcome {} - BlueHost.com</title>",
    "{} is parked</title>",
  ];
  for s in &domain_strings {
    if html.contains(&s.replace("{}", domain)) {
      return true;
    }
  }
  // if html.contains("domain") && html.contains("is parked") {
  //   return true;
  // }
  false
}

pub fn check_lol(html: &str) -> bool {
  html.contains("domain") && html.contains("is parked")
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn fixture_html() {
    let cases = vec![
      ("foo.com", "godaddy.html", true),
      ("foo.com", "netrivet.html", false),
      ("foo.com", "domainnameshop.html", true),
      ("foo.com", "oceanbamk.html", true),
      ("foo.com", "venapartments.html", true),
      ("foo.com", "german.html", true),
      ("foo.com", "timberlandjoineryltd.html", true),
      ("foo.com", "epik.html", true),
      ("foo.com", "summitjourneys.html", true),
      ("foo.com", "egress.html", true),
      ("loco-capital.com", "loco-capital.html", true),
      ("danielatraub.com", "danielatraub.html", true),
      ("indocinrx.com", "indocinrx.html", true),
      ("offeringtalks.com", "offeringtalks.html", true),
    ];
    for (domain, file, expected) in cases {
      let filepath = format!("parked_html/{}", file);
      let html = std::fs::read_to_string(filepath).unwrap();
      assert_eq!(check(domain, &html), expected, "file: `{}`", file);
    }
  }

  #[test]
  fn input_html() {
    let cases = vec![
      (
        "foo.com",
        "<title>Welcome foo.com - BlueHost.com</title>",
        true,
      ),
      ("foo.com", "<title>www.foo.com is parked</title>", true),
    ];
    for (domain, html, expected) in cases {
      assert_eq!(check(domain, html), expected, "input: `{}`", html);
    }
  }
}
