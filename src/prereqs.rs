use semver::{Version, VersionReq};
use std::process::{Command, Stdio};

use crate::internal::*;

pub fn check() -> Result<()> {
  check_bun_version()
}

fn check_bun_version() -> Result<()> {
  let bun_bin = std::env::var("BUN_BIN")
    .expect("missing required env var: `BUN_BIN`, see `.env.example`");

  let bun_version = Command::new(bun_bin)
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
