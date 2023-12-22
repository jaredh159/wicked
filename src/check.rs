// reqwest docs: https://docs.rs/reqwest/0.10.7/reqwest/

pub async fn domain(domain: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("https://{}", domain);
  let response = reqwest::get(&url).await?;
  if !response.status().is_success() {
    todo!("handle bad res");
  }
  let body = response.text().await?;
  println!("body = \n\n{:?}", body);
  Ok(())
}
