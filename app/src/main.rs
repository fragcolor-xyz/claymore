use std::time::{Duration, Instant};

use chainblocks::cbl_env;
use chainblocks::cblisp::new_env;
use chainblocks::core::{init, sleep};
use chainblocks::types::{ClonedVar, Node};

const QUANTUM_TIME: f64 = 1.0 / 120.0;

fn main() {
  init();

  let env = new_env();
  let main_node = Node::default();
  let main_chain = cbl_env!(env, concat!("(do ", include_str!("main.edn"), ")")).unwrap();
  main_node.schedule(main_chain.0.try_into().unwrap());

  let quantum = Duration::from_secs_f64(QUANTUM_TIME);
  let zero = Duration::from_secs_f64(0.0);
  let mut now = Instant::now();
  let mut next = now + quantum;
  loop {
    if !main_node.tick() {
      break;
    }

    // TODO manipulate node/chains etc

    // We are done, balance time to make it as real time as possible
    now = Instant::now();
    let real_sleep = next - now;
    if real_sleep < zero {
      // We are behind, don't sleep
      next = now + quantum;
    } else {
      // We are ahead, sleep
      next = now + real_sleep;
      sleep(real_sleep.as_secs_f64());
    }
  }
}
