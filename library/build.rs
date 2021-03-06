use cmake::Config;
use std::env;

fn main() {
  let mut dst = Config::new("../../shards");

  dst.build_target("shards-dll");

  #[cfg(target_os = "linux")]
  dst.define("USE_FPIC", "1");

  let dst = dst.build();

  println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
  println!("cargo:rustc-link-search=native={}/build", dst.display());
  println!("cargo:rustc-link-lib=dylib=shards");
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/");

  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let mut config: cbindgen::Config = Default::default();
  config.language = cbindgen::Language::C;
  config.sys_includes = vec!["shards.h".to_string()];
  config.cpp_compat = true;
  config.after_includes = Some(
    r#"
#define ExternalVar struct SHVar
#define Var struct SHVar"#
      .to_string(),
  );
  cbindgen::generate_with_config(&crate_dir, config)
    .unwrap()
    .write_to_file("target/claymore.h");
}
