use crate::internal::*;

pub async fn check(
  base_url: &str,
  content: &[html::Content],
  http: &HttpClient,
  result: &mut TestResult,
) {
  let all_srcs = content.iter().filter_map(|c| match c {
    html::Content::ImgSrc(src) => Some(src),
    _ => None,
  });
  result.num_total_images = all_srcs.clone().unique().count();

  let filtered_srcs = all_srcs
    .filter(|src| !src.contains(".gif") && !src.contains(".svg"))
    .unique()
    .map(|src| absolute_url(base_url, src));

  let mut num_sexy_imgs = 0;
  let mut num_porn_imgs = 0;

  for url in filtered_srcs {
    result.num_images_tested += 1;
    let Ok(response) = http.get(url.as_ref()).send().await else {
      log::error!("http request to `{url}` failed with error");
      continue;
    };

    if !response.status().is_success() {
      log::debug!(
        "download req to `{url}` failed with status {}",
        response.status()
      );
      continue;
    }

    let Ok(bytes) = response.bytes().await else {
      log::error!("failed to read bytes from response to `{url}`");
      continue;
    };

    let filename = format!("{}.dat", Uuid::new_v4());
    let filepath = format!("images/{filename}");
    if std::fs::write(&filepath, bytes).is_err() {
      log::error!("failed to write image from url {url} to disk");
      continue;
    }

    if let Some(Classification { porn, hentai, sexy }) = classify(&filename, http).await {
      if porn > 0.85 || hentai > 0.85 {
        log::debug!("image found to be PORN: {url}");
        num_porn_imgs += 1;
      } else if sexy > 0.9 {
        log::debug!("image found to be SEXY: {url}");
        num_sexy_imgs += 1;
      } else {
        log::trace!("image found to be SAFE: {url}");
      }
      if num_porn_imgs > 1 || num_sexy_imgs > 3 {
        log::info!("site found to be PORN by IMAGE check: {base_url}");
        result.is_porn = true;
        let _ = std::fs::remove_file(&filepath);
        return;
      }
    };
    let _ = std::fs::remove_file(&filepath);
  }
  log::trace!("finished checking images at {base_url}");
}

async fn classify(filename: &str, http: &HttpClient) -> Option<Classification> {
  let url = format!("http://localhost:8484/{}", filename);
  let Ok(response) = http.get(&url).send().await else {
    log::error!("http request to `{url}` failed with error");
    return None;
  };

  if !response.status().is_success() {
    return None;
  }
  let Ok(bytes) = response.bytes().await else {
    return None;
  };

  match serde_json::from_slice(&bytes) {
    Ok(classification) => classification,
    Err(err) => {
      log::error!("failed to decode classification from response: {}", err);
      None
    }
  }
}

#[derive(Debug, Deserialize)]
struct Classification {
  porn: f64,
  hentai: f64,
  sexy: f64,
}

fn absolute_url<'a>(base_url: &str, src: &'a str) -> Cow<'a, str> {
  if src.starts_with("http") {
    Cow::Borrowed(src)
  } else {
    let slash = if src.starts_with('/') { "" } else { "/" };
    Cow::Owned(format!("{}{}{}", base_url, slash, src))
  }
}

pub fn start_server() -> Result<std::process::Child> {
  let bun_bin = std::env::var("BUN_BIN").unwrap();
  let proc = std::process::Command::new(bun_bin)
    .arg("index.ts")
    .spawn()?;
  // give time for server to start
  std::thread::sleep(Duration::from_millis(500)); // todo, more for prod
  Ok(proc)
}

pub fn cleanup(mut server_proc: std::process::Child) -> Result<()> {
  std::fs::remove_dir_all("images")?;
  std::fs::create_dir("images")?;
  std::fs::write("images/.gitkeep", "")?;
  server_proc.kill()?;
  Ok(())
}
