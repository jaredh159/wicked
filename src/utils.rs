#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub fn percent(part: u32, total: u32) -> f32 {
  if total == 0 {
    0.0
  } else {
    (part as f32 / total as f32) * 100.0
  }
}

pub fn percent_str(part: u32, total: u32) -> String {
  format!("{:.2}", percent(part, total))
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn test_percent() {
  assert_eq!(percent(0, 0) as i8, 0);
  assert_eq!(percent(0, 1) as i8, 0);
  assert_eq!(percent(1, 1) as i8, 100);
  assert_eq!(percent(1, 2) as i8, 50);
  assert_eq!(percent(2, 8) as i8, 25);
}
