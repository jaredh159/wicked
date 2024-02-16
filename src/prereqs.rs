use semver::{Version, VersionReq};
use std::process::{Command, Stdio};

use crate::internal::*;

pub fn check(config: &Config) -> Result<()> {
  check_bun_version(config)
}

fn check_bun_version(config: &Config) -> Result<()> {
  let bun_version = Command::new(&config.bun_bin_path)
    .arg("--version")
    .stdout(Stdio::piped())
    .spawn()?;

  let output = bun_version.wait_with_output().unwrap();
  let semver = String::from_utf8_lossy(&output.stdout).to_string();
  let req = VersionReq::parse(">=1.0.21").unwrap();
  let version = Version::parse(semver.trim())?;

  if !req.matches(&version) {
    return Err(e(format!(
      "requires `bun` version {}, found: {}, try `bun upgrade`",
      req, version
    )));
  }

  Ok(())
}
