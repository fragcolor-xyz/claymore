extern crate chainblocks;

use chainblocks::cbl_env;
use chainblocks::types::ClonedVar;

#[test]
fn main() {
  chainblocks::core::init();
  let res = cbl_env!(include_str!("test.edn")).unwrap();
  let res = <i64>::try_from(&res.0).unwrap();
  assert_eq!(res, 99);
}
