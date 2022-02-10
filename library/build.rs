use cmake::Config;
use std::env;

fn main() {
  let mut dst = Config::new("../../chainblocks");

  dst.build_target("cbl-dll");

  #[cfg(target_os = "linux")]
  dst.define("USE_FPIC", "1");

  let dst = dst.build();

  println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
  println!("cargo:rustc-link-search=native={}/build", dst.display());
  println!("cargo:rustc-link-lib=dylib=cbl");

  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let mut config: cbindgen::Config = Default::default();
  config.language = cbindgen::Language::C;
  config.sys_includes = vec!["chainblocks.h".to_string()];
  config.cpp_compat = true;
  config.after_includes = Some(
    r#"
#define ExternalVar struct CBVar
#define Var struct CBVar"#
      .to_string(),
  );
  cbindgen::generate_with_config(&crate_dir, config)
    .unwrap()
    .write_to_file("target/claymore.h");
}
