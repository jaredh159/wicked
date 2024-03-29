use crate::internal::*;

pub async fn check(
  base_url: &str,
  content: &[html::Content],
  config: &Config,
  result: &mut TestResult,
) {
  let download_client = http::client(redirect::Policy::limited(2), 4);
  let classify_client = http::client(redirect::Policy::none(), 2);

  let all_srcs = content.iter().filter_map(|c| match c {
    html::Content::ImgSrc(src) => Some(src),
    _ => None,
  });
  result.num_total_images = all_srcs.clone().unique().count();

  let filtered_srcs = all_srcs
    .filter(|src| {
      !src.contains(".gif") && !src.contains(".svg") && !src.contains("data:")
    })
    .unique()
    .map(|src| absolute_url(base_url, src));

  let mut num_sexy_imgs = 0;
  let mut num_porn_imgs = 0;
  let mut num_failed = 0;

  for url in filtered_srcs.take(50) {
    result.num_images_tested += 1;

    let response = match download_client.get(url.as_ref()).send().await {
      Ok(response) => response,
      Err(err) => {
        num_failed += 1;
        log::debug!("http request to `{url}` failed with error={err}");
        if num_failed > 10 {
          log::warn!(
            "bailing early from {base_url} image check: too many failed requests"
          );
          return;
        }
        continue;
      }
    };

    if !response.status().is_success() {
      log::debug!(
        "download req to `{url}` failed with status {}",
        response.status()
      );
      continue;
    }

    let bytes = match response.bytes().await {
      Ok(bytes) => bytes,
      Err(err) => {
        log::debug!("failed to read bytes from response to `{url}`: {err}");
        continue;
      }
    };

    if bytes.len() < 1_024 {
      log::trace!("image {url} is too small to classify ({}B)", bytes.len());
      continue;
    }

    let filename = format!("{}.dat", Uuid::new_v4());
    let filepath = format!("images/{filename}");
    if std::fs::write(&filepath, bytes).is_err() {
      log::error!("failed to write image from url {url} to disk");
      continue;
    }

    if let Some(Classification { porn, hentai, sexy }) =
      classify(&filename, &classify_client).await
    {
      if porn > config.porn_image_confidence || hentai > config.hentai_image_confidence {
        log::debug!("image found to be PORN: {url}");
        num_porn_imgs += 1;
      } else if sexy > config.sexy_image_confidence {
        log::debug!("image found to be SEXY: {url}");
        num_sexy_imgs += 1;
      } else {
        log::trace!("image found to be SAFE: {url}");
      }
      if num_porn_imgs > config.num_porn_images_threshold
        || num_sexy_imgs > config.num_sexy_images_threshold
        || (num_porn_imgs + num_sexy_imgs > config.num_sexy_images_threshold)
      {
        log::error!("site found to be PORN by IMAGE check: {base_url}");
        result.is_porn = true;
        _ = std::fs::remove_file(&filepath);
        return;
      }
    };
    _ = std::fs::remove_file(&filepath);
  }
  log::trace!("finished checking images at {base_url}");
}

async fn classify(filename: &str, http: &HttpClient) -> Option<Classification> {
  let url = format!("http://localhost:8484/{}", filename);
  let response = match http.get(&url).send().await {
    Ok(response) => response,
    Err(err) => {
      log::error!("http request to `{url}` failed with error={err}");
      return None;
    }
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

#[derive(Debug, serde::Deserialize)]
struct Classification {
  porn: f32,
  hentai: f32,
  sexy: f32,
}

fn absolute_url<'a>(base_url: &str, src: &'a str) -> Cow<'a, str> {
  if src.starts_with("http") {
    Cow::Borrowed(src)
  } else if src.starts_with("//") {
    if base_url.starts_with("https") {
      Cow::Owned(format!("https:{}", src))
    } else {
      Cow::Owned(format!("http:{}", src))
    }
  } else {
    let slash = if src.starts_with('/') { "" } else { "/" };
    Cow::Owned(format!("{}{}{}", base_url, slash, src))
  }
}

pub fn start_server(config: &Config) -> Result<std::process::Child> {
  let proc = std::process::Command::new(&config.bun_bin_path)
    .arg("index.ts")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .spawn()?;
  // give time for server to start
  std::thread::sleep(Duration::from_millis(config.image_server_startup_wait_ms));
  Ok(proc)
}

pub fn cleanup(mut server_proc: std::process::Child) -> Result<()> {
  std::fs::remove_dir_all("images")?;
  std::fs::create_dir("images")?;
  std::fs::write("images/.gitkeep", "")?;
  server_proc.kill()?;
  Ok(())
}
