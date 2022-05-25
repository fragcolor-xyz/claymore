use std::time::{Duration, Instant};

use chainblocks::cbl_env;
use chainblocks::cblisp::new_env;
use chainblocks::core::{init, sleep};
use chainblocks::types::{ClonedVar, Node, Table, Chain, BlockRef};
use claylib::program::Program;
use claylib::ProtoTrait;

const QUANTUM_TIME: f64 = 1.0 / 120.0;

fn main() {
  init();

  let env = new_env();
  let main_node = Node::default();

  // exec the script, notice we on purpose return nil to avoid varifying the trait inside!
  // the reason being, varify will consume the blocks and null them out, which is not what we want
  let _ = cbl_env!(env, concat!("(do ", include_str!("main.edn"), " nil)")).expect("main.edn");
  // extract the trait we need, this step consumes the blocks from the script env, Program will be invalid after!
  let program: ClonedVar = cbl_env!(env, "(do (println Program) Program)").expect("Program ClonedVar");
  let program: Table = program.0.as_ref().try_into().expect("Program Table");
  let program = Program::distill(&program).expect("Program distill");

  let main_chain = Chain::default();
  main_chain.set_name("Main Claymore Chain");

  if let Some(on_rez) = program.on_rez {
    for block in on_rez {
      main_chain.add_block(block.try_into().expect("OnRez Block"));
    }
  }

  let looped_chain = Chain::default();
  looped_chain.set_name("Claymore Main Loop Chain");
  looped_chain.set_looped(true);

  for block in program.advance {
    looped_chain.add_block(block.try_into().expect("Advance Block"));
  }

  let dispatch_loop = BlockRef::create("Dispatch");
  dispatch_loop.set_parameter(0, looped_chain.0.into());

  main_chain.add_block(dispatch_loop);

  if let Some(on_derez) = program.on_derez {
    for block in on_derez {
      main_chain.add_block(block.try_into().expect("OnDerez Block"));
    }
  }

  main_node.schedule(main_chain.0);

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
