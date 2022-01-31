use chainblocks::{
  cbl, cbl_env,
  cblisp::{new_env, new_sub_env},
  core::cloneVar,
  types::{ChainRef, ClonedVar, ExternalVar, Node, Table, Var},
};
use edn_rs::{edn, Edn, Map};
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
  INIT.call_once(|| {
    chainblocks::core::init();
  });
}

// TODO, the following are fully C API calls.. we should instead implement them in pure rust so to use
// the same code for both C and Rust and rust side could become a command line tool to inspect fragments

pub enum PollState {
  Running,
  Failed(String),
  Finished(ClonedVar),
}

pub struct GetDataRequest {
  pub chain: ChainRef,
  pub hash: ExternalVar,
  pub timeout: ExternalVar,
  pub result: ExternalVar,
}

pub fn start_get_data(fragment_hash: [u8; 32]) -> GetDataRequest {
  initialize();

  let chain = cbl!(include_str!("fetch-fragment.edn")).unwrap();
  let chain = <ChainRef>::try_from(chain.0).unwrap();

  let hash = fragment_hash[..].into();
  chain.set_external("hash", &hash);

  let timeout = 30i32.into();
  chain.set_external("timeout", &timeout);

  let result: [u8; 0] = [];
  let result = result[..].into();
  chain.set_external("result", &result);

  GetDataRequest {
    chain,
    hash,
    timeout,
    result,
  }
}

// C bindings

#[no_mangle]
pub extern "C" fn clmrGetDataStart(fragment_hash: *const u8) -> *mut GetDataRequest {
  let hash = unsafe { std::slice::from_raw_parts(fragment_hash, 32) };
  let hash_fixed: [u8; 32] = hash.try_into().unwrap();
  let boxed = Box::new(start_get_data(hash_fixed));
  Box::into_raw(boxed)
}

pub fn poll_chain(chain: ChainRef) -> PollState {
  match chain.get_result() {
    Ok(result) => {
      if let Some(result) = result {
        PollState::Finished(result)
      } else {
        PollState::Running
      }
    }
    Err(err) => {
      PollState::Failed(err.to_string())
    }
  }
}

// C bindings

#[no_mangle]
pub extern "C" fn clmrGetDataPoll(request: *mut GetDataRequest, output: *mut Var) -> PollState {
  // let request = unsafe { Box::from_raw(request) };
  let chain = unsafe { (*request).chain };
  poll_chain(chain)
}

#[test]
fn chain() {
  initialize();

  let chain = cbl!(include_str!("test-chain.edn")).unwrap();
  let chain = <ChainRef>::try_from(chain.0).unwrap();
  let variable = 10i32.into();
  let result = 0i32.into();
  chain.set_external("extern1", &variable);
  chain.set_external("result", &result);
  let node = Node::default();
  node.schedule(chain);
  assert!(node.tick());
  let res = <i64>::try_from(&result.0).unwrap();
  assert_eq!(res, 20i64);
}

#[test]
fn main() {
  initialize();

  let res = cbl!(include_str!("test-simple.edn")).unwrap();
  let res = <i64>::try_from(&res.0).unwrap();
  assert_eq!(res, 99);
}

#[test]
fn edn_rs_usage() {
  initialize();

  let edn = edn!({:a 1 :b 2 :c 3});
  let edn = edn.to_string();
  assert_eq!(edn, "{:a 1, :b 2, :c 3, }");
  let res = cbl!(edn).unwrap();
  let mut res = <Table>::try_from(&res.0).unwrap();
  let a = <i64>::try_from(res.get_fast_static("a\0")).unwrap();
  assert_eq!(a, 1);
  let b = <i64>::try_from(res.get_fast_static("b\0")).unwrap();
  assert_eq!(b, 2);
  let c = <i64>::try_from(res.get_fast_static("c\0")).unwrap();
  assert_eq!(c, 3);
}

#[test]
fn sub_envs() {
  initialize();

  let root = new_env();
  let sub1 = new_sub_env(&root);
  let sub2 = new_sub_env(&root);
  let sub3 = new_sub_env(&root);
  cbl_env!(root, "(def x 10)");
  let x = cbl_env!(sub1, "(do (deflocal! y 11) x)").unwrap();
  let x = <i64>::try_from(&x.0).unwrap();
  assert_eq!(x, 10);
  let x = cbl_env!(sub2, "(do (deflocal! y 12) x)").unwrap();
  let x = <i64>::try_from(&x.0).unwrap();
  assert_eq!(x, 10);
  let x = cbl_env!(sub3, "(do (deflocal! y 13) x)").unwrap();
  let x = <i64>::try_from(&x.0).unwrap();
  assert_eq!(x, 10);
  let y = cbl_env!(sub1, "y").unwrap();
  let y = <i64>::try_from(&y.0).unwrap();
  assert_eq!(y, 11);
  let y = cbl_env!(sub2, "y").unwrap();
  let y = <i64>::try_from(&y.0).unwrap();
  assert_eq!(y, 12);
  let y = cbl_env!(sub3, "y").unwrap();
  let y = <i64>::try_from(&y.0).unwrap();
  assert_eq!(y, 13);
}
