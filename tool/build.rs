fn main() {
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/");
}
