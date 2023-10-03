use chrono::{SecondsFormat, Utc};

fn main() {
    let build_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    println!("cargo:rustc-env=BUILD_DATE={build_date}");
}
