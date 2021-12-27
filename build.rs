use cmake::Config;

fn main() {
  let dst = Config::new("../chainblocks")
    .build_target("cbl-dll")
    .build();
  println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
  println!("cargo:rustc-link-lib=dylib=cbl");
}
