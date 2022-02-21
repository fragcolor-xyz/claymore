use chainblocks::{
  cbl, cbl_env,
  cblisp::{new_env, new_sub_env, ScriptEnv},
  core::cloneVar,
  types::{ChainRef, ClonedVar, ExternalVar, Node, Table, Var},
};
use edn_rs::{edn, Edn, Map};
use std::sync::Once;
use std::{thread, time};

static INIT: Once = Once::new();

pub fn initialize() {
  INIT.call_once(|| {
    chainblocks::core::init();
  });
}

#[repr(u8)]
pub enum PollState {
  Running,
  Failed(ClonedVar),
  Finished(ClonedVar),
}

#[repr(C)]
#[derive(Default)]
pub struct GetDataRequest {
  pub chain: ClonedVar,
  pub hash: ExternalVar,
  pub result: ExternalVar,
  pub env: Option<ScriptEnv>,
}

pub fn start_get_data(fragment_hash: [u8; 32]) -> Box<GetDataRequest> {
  initialize();

  let root = new_env();

  let mut request = Box::new(GetDataRequest::default());

  request.chain = cbl_env!(root, include_str!("proto-fetch.edn")).unwrap();
  let chain = <ChainRef>::try_from(request.chain.0).unwrap();

  request.hash = fragment_hash[..].into();
  chain.set_external("hash", &mut request.hash);

  let result = Table::new();
  request.result = (&result).into();
  chain.set_external("result", &mut request.result);

  request.env = Some(root);

  request
}

// C bindings

#[no_mangle]
pub extern "C" fn clmrGetDataStart(fragment_hash: *const u8) -> *mut GetDataRequest {
  let hash = unsafe { std::slice::from_raw_parts(fragment_hash, 32) };
  let hash_fixed: [u8; 32] = hash.try_into().unwrap();
  let boxed = start_get_data(hash_fixed);
  Box::into_raw(boxed)
}

#[no_mangle]
pub extern "C" fn clmrGetDataFree(request: *mut GetDataRequest) {
  unsafe {
    Box::from_raw(request);
  }
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
      let err: ClonedVar = err.into();
      PollState::Failed(err)
    }
  }
}

// C bindings

#[no_mangle]
pub extern "C" fn clmrPoll(chain: ChainRef, output: *mut *mut PollState) -> bool {
  match poll_chain(chain) {
    PollState::Running => false,
    PollState::Failed(err) => {
      unsafe {
        *output = Box::into_raw(Box::new(PollState::Failed(err)));
      }
      true
    }
    PollState::Finished(result) => {
      unsafe {
        *output = Box::into_raw(Box::new(PollState::Finished(result)));
      }
      true
    }
  }
}

#[no_mangle]
pub extern "C" fn clmrPollFree(state: *mut PollState) {
  unsafe {
    Box::from_raw(state);
  }
}

pub fn proto_upload(
  node_url: &str,
  signer_key: &str,
  auth_key: &str,
  proto_type: &str,
  data: Table,
) -> Result<(), &'static str> {
  initialize();

  let root = new_env();

  cbl_env!(root, concat!("(do ", include_str!("proto-common.edn"), ")")).unwrap();

  let chain =
    cbl_env!(root, include_str!("proto-upload.edn")).expect("proto-upload script processing");
  let chain = <ChainRef>::try_from(chain.0).expect("proto-upload chain");

  let mut node: ExternalVar = node_url.into();
  chain.set_external("rpc-server", &mut node);

  let mut signer_key: ExternalVar = signer_key.into();
  chain.set_external("signer-key", &mut signer_key);

  let mut auth_key: ExternalVar = auth_key.into();
  chain.set_external("auth-key", &mut auth_key);

  let mut type_: ExternalVar = proto_type.into();
  chain.set_external("type", &mut type_);

  let mut data: ExternalVar = (&data).into();
  chain.set_external("data", &mut data);

  let node = Node::default();
  node.schedule(chain);

  loop {
    if !node.tick() {
      break;
    }
  }

  Ok(())
}

#[test]
fn chain() {
  initialize();

  let chain = cbl!(include_str!("test-chain.edn")).unwrap();
  let chain = <ChainRef>::try_from(chain.0).unwrap();
  let mut variable = 10i32.into();
  let mut result = 0i32.into();
  chain.set_external("extern1", &mut variable);
  chain.set_external("result", &mut result);
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

#[test]
fn fragment_get_data_test() {
  initialize();

  let mut hash: [u8; 32] = [0; 32];
  hex::decode_to_slice(
    "953f867f5e7af34b031d2689ea1486420571dfac0cd4043b173b0035e621c0dd",
    &mut hash,
  )
  .unwrap();

  let request = start_get_data(hash);
  let chain = <ChainRef>::try_from(request.chain.0).unwrap();

  let node = Node::default();
  node.schedule(chain);

  loop {
    node.tick();
    let status = poll_chain(chain);
    match status {
      PollState::Finished(result) => {
        let result = <&[u8]>::try_from(&result.0).unwrap();
        assert_eq!(result, b"");
        break;
      }
      PollState::Failed(err) => {
        let err = <&str>::try_from(&err.0).unwrap();
        panic!("{}", err);
      }
      PollState::Running => {
        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);
      }
    }
  }
}
