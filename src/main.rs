use std::{cmp, fs};
use std::fs::{File, read_dir};
use std::io::Write;
use std::num::ParseIntError;

use clap::Parser;
use ddc::Ddc;
use eyre::{eyre, Result};

fn parse_feature_code(input: &str) -> Result<u8, ParseIntError> {
  if let Some(s) = input.strip_prefix("0x") {
    u8::from_str_radix(s, 16)
  } else if let Some(s) = input.strip_suffix(&['h', 'H']) {
    u8::from_str_radix(s, 16)
  } else {
    input.parse()
  }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
  /// output name such as DP-1
  output_name: String,
  /// feature code in decimal or 0xFF or FFh format
  #[clap(value_parser = parse_feature_code)]
  feature_code: u8,
  /// value to be set; when not present show current value
  feature_value: Option<String>,
}

// /sys/class/drm/card*-{name}/i2c-*
fn get_i2c_dev(output: &str) -> Result<String> {
  let mut dev = None;
  for entry in read_dir("/sys/class/i2c-dev").unwrap() {
    let path = entry.unwrap().path();
    let link_path = path.read_link().unwrap();
    let name = path.file_name().unwrap().to_str().unwrap();
    let points_to = link_path.to_str().unwrap();
    let points_to_split = points_to.split('/').collect::<Vec<_>>();
    for item in points_to_split {
      if item.starts_with("card") && item.ends_with(output) {
        dev = Some(name.to_string());
        break;
      }
    }
    if dev.is_some() {
      break;
    }
  };

  dev.ok_or_else(|| eyre!("i2c dev not found"))
}

fn set_value(ddc: &mut ddc_i2c::I2cDeviceDdc, feature_code: u8, value: String) -> Result<()> {
  let current = ddc.get_vcp_feature(feature_code)?;
  let current_value = current.value();
  let max = current.maximum();
  let mut new_value = current_value;
  if value.ends_with('+') || value.ends_with('-') {
    let relative = value.trim_end_matches(|c| c == '+' || c == '-');
    let relative = relative.parse::<u16>()?;
    // make sure we don't go below 0
    if value.ends_with("-") {
      new_value = current_value.saturating_sub(relative);
    }
    // make sure we don't go above max
    if value.ends_with("+") {
      new_value = cmp::min(max, current_value + relative)
    }
  } else {
    new_value = value.parse::<u16>()?;
    new_value = if new_value > max { max } else { new_value };
  }
  ddc.set_vcp_feature(feature_code, new_value)?;

  Ok(())
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  let output_name = cli.output_name;
  let i2c_name = if output_name.starts_with("i2c-") {
    output_name.clone()
  } else {
    get_i2c_dev(&output_name)?
  };
  let dev = format!("/dev/{}", i2c_name);
  let mut ddc = ddc_i2c::from_i2c_device(dev).unwrap();

  if let Some(v) = cli.feature_value {
    set_value(&mut ddc, cli.feature_code, v)?;
  } else {
    ddc.get_vcp_feature(cli.feature_code)?;
  }

  let current = ddc.get_vcp_feature(cli.feature_code)?;
  let output_json = format!("{{\"value\":{}, \"percentage\":{:.0}, \"max\":{}}}",
                            current.value(),
                            (current.value() as f32 / current.maximum() as f32) * 100.0,
                            current.maximum());
  fs::create_dir_all("/tmp/backlight")?;
  let mut file = File::create(format!("/tmp/backlight/{}.json", output_name))?;
  write!(file, "{}", output_json)?;

  println!("{}", output_json);

  Ok(())
}
