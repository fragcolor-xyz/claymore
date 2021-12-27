extern crate chainblocks;
extern crate edn_rs;

use chainblocks::cbl_env;
use chainblocks::types::{ClonedVar, Table, Var};
use edn_rs::{edn, Edn, Map};
use std::fs;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
  INIT.call_once(|| {
    chainblocks::core::init();
  });
}

#[test]
fn main() {
  initialize();

  let res = cbl_env!(include_str!("test.edn")).unwrap();
  let res = <i64>::try_from(&res.0).unwrap();
  assert_eq!(res, 99);
}

#[test]
fn edn_rs_usage() {
  initialize();

  let edn = edn!({:a 1 :b 2 :c 3});
  let edn = edn.to_string();
  assert_eq!(edn, "{:a 1, :b 2, :c 3, }");
  let res = cbl_env!(edn).unwrap();
  let mut res = <Table>::try_from(&res.0).unwrap();
  let a = <i64>::try_from(res.get_fast_static("a\0")).unwrap();
  assert_eq!(a, 1);
  let b = <i64>::try_from(res.get_fast_static("b\0")).unwrap();
  assert_eq!(b, 2);
  let c = <i64>::try_from(res.get_fast_static("c\0")).unwrap();
  assert_eq!(c, 3);
}

#[no_mangle]
pub extern "C" fn clmr_load_file(path: *const u8, path_len: usize, output: *mut Var) -> bool {
  initialize();

  let path = unsafe { std::slice::from_raw_parts(path, path_len) };
  let path = std::str::from_utf8(path).expect("Failed to convert path to utf8");
  let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
  let res = cbl_env!(contents);
  if let Some(res) = res {
    unsafe {
      *output = res.0;
    }
    true
  } else {
    false
  }
}
