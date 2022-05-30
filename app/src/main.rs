use std::time::{Duration, Instant};

use shards::shards_env;
use shards::scripting::new_env;
use shards::core::{init, sleep};
use shards::types::{ClonedVar, Mesh, WireRef};

const QUANTUM_TIME: f64 = 1.0 / 120.0;

fn main() {
  init();

  let env = new_env();
  let main_mesh = Mesh::default();

  // exec the script, notice we on purpose return nil to avoid varifying the trait inside!
  // the reason being, varify will consume the shards and null them out, which is not what we want
  let _ = shards_env!(env, concat!("(do ", include_str!("main.edn"), " nil)")).expect("main.edn");
  let loop_wire = shards_env!(env, include_str!("loop.edn")).expect("loop.edn");
  let loop_wire: WireRef = loop_wire.0.try_into().expect("loop.edn WireRef");
  main_mesh.schedule(loop_wire);

  let quantum = Duration::from_secs_f64(QUANTUM_TIME);
  let zero = Duration::from_secs_f64(0.0);
  let mut now = Instant::now();
  let mut next = now + quantum;
  loop {
    if !main_mesh.tick() {
      break;
    }

    // TODO manipulate mesh/wires etc

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
