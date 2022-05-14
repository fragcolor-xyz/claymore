use chainblocks::types::{Node, ClonedVar};
use chainblocks::core::{init, sleep};
use chainblocks::cblisp::{new_env};
use chainblocks::cbl_env;

const QUANTUM_TIME: f64 = 1.0 / 120.0;

fn main() {
  init();

  let env = new_env();
  let main_node = Node::default();
  let main_chain = cbl_env!(env, include_str!("main.edn")).unwrap();
  main_node.schedule(main_chain.0.try_into().unwrap());

  loop {
    if !main_node.tick() {
      break;
    }

    // TODO manipulated node/chains etc

    // TODO make this perfect timing (we have that in mal)
    sleep(QUANTUM_TIME);
  }
}
