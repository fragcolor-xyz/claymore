use cmake::Config;
use std::env;

fn main() {
  let dst = Config::new("../chainblocks")
    .build_target("cbl-dll")
    .build();
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
#define ExternalVar CBVar
#define Var CBVar"#
      .to_string(),
  );
  cbindgen::generate_with_config(&crate_dir, config)
    .unwrap()
    .write_to_file("target/claymore.h");
}
