extern crate chainblocks;

use chainblocks::cbl_env;

#[test]
fn main() {
  chainblocks::core::init();
  cbl_env!("(println \"Hello, world!\")");
}
