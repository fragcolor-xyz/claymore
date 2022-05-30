use shards::{
  shards, shards_env,
  scripting::{new_env, new_sub_env, ScriptEnv},
  core::cloneVar,
  types::{WireRef, ClonedVar, ExternalVar, Mesh, Table, Var},
};
use edn_rs::{edn, Edn, Map};
use std::sync::Once;
use std::{thread, time};

pub mod program;

pub trait ProtoTrait {
  fn distill(traits: &Table) -> Result<Self, &'static str>
  where
    Self: Sized;
}

static INIT: Once = Once::new();

pub fn initialize() {
  INIT.call_once(|| {
    shards::core::init();
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
  pub wire: ClonedVar,
  pub hash: ExternalVar,
  pub result: ExternalVar,
  pub env: Option<ScriptEnv>,
}

#[repr(C)]
#[derive(Default)]
pub struct UploadRequest {
  pub wire: ClonedVar,
  pub node: ExternalVar,
  pub signer_key: ExternalVar,
  pub auth_key: ExternalVar,
  pub proto_type: ExternalVar,
  pub data: ExternalVar,
  pub env: Option<ScriptEnv>,
}

pub fn start_get_data(fragment_hash: [u8; 32]) -> Box<GetDataRequest> {
  initialize();

  let root = new_env();

  let mut request = Box::new(GetDataRequest::default());

  request.wire = shards_env!(root, include_str!("proto-fetch.edn")).unwrap();
  let wire = <WireRef>::try_from(request.wire.0).unwrap();

  request.hash = fragment_hash[..].into();
  wire.set_external("hash", &mut request.hash);

  let result = Table::new();
  request.result = (&result).into();
  wire.set_external("result", &mut request.result);

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

pub fn poll_chain(wire: WireRef) -> PollState {
  match wire.get_result() {
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
pub extern "C" fn clmrPoll(wire: WireRef, output: *mut *mut PollState) -> bool {
  match poll_chain(wire) {
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

pub fn start_proto_upload(
  node_url: &str,
  signer_key: &str,
  proto_type: &str,
  data: Table,
) -> Box<UploadRequest> {
  initialize();

  let root = new_env();

  let mut request = Box::new(UploadRequest::default());

  shards_env!(root, concat!("(do ", include_str!("proto-common.edn"), ")")).unwrap();

  request.wire =
    shards_env!(root, include_str!("proto-upload.edn")).expect("proto-upload script processing");
  let wire = <WireRef>::try_from(request.wire.0).expect("proto-upload wire");

  request.node = node_url.into();
  wire.set_external("rpc-server", &mut request.node);

  request.signer_key = signer_key.into();
  wire.set_external("signer-key", &mut request.signer_key);

  request.proto_type = proto_type.into();
  wire.set_external("type", &mut request.proto_type);

  request.data = (&data).into();
  wire.set_external("data", &mut request.data);

  request.env = Some(root);

  request
}

pub fn proto_upload(
  node_url: &str,
  signer_key: &str,
  proto_type: &str,
  data: Table,
) -> Result<(), &'static str> {
  let request = start_proto_upload(node_url, signer_key, proto_type, data);
  let wire = <WireRef>::try_from(request.wire.0).unwrap();

  let mesh = Mesh::default();
  mesh.schedule(wire);

  loop {
    mesh.tick();
    let status = poll_chain(wire);
    match status {
      PollState::Finished(_) => {
        break;
      }
      PollState::Failed(err) => {
        let err = <&str>::try_from(&err.0).unwrap();
        panic!("{}", err);
      }
      PollState::Running => {
        continue;
      }
    }
  }

  Ok(())
}

#[no_mangle]
pub extern "C" fn clmrUpload(var: *const Var) -> *mut UploadRequest {
  let table: Table;
  unsafe {
    table = (&*var).try_into().unwrap();
  }

  // FIXME hardcoded values
  let node = "http://127.0.0.1:9933";
  let signer = "//Dave";
  let key = "//Alice";
  // FIXME extract it from table or modify the EDN code to do the same
  let type_ = "audio";

  // TODO add checks to make sure the table is well-formed

  let boxed = start_proto_upload(&node, &signer, &type_, table);
  Box::into_raw(boxed)
}

#[no_mangle]
pub extern "C" fn clmrUploadFree(request: *mut UploadRequest) {
  unsafe {
    Box::from_raw(request);
  }
}

#[test]
fn wire() {
  initialize();

  let wire = shards!(include_str!("test-wire.edn")).unwrap();
  let wire = <WireRef>::try_from(wire.0).unwrap();
  let mut variable = 10i32.into();
  let mut result = 0i32.into();
  wire.set_external("extern1", &mut variable);
  wire.set_external("result", &mut result);
  let mesh = Mesh::default();
  mesh.schedule(wire);
  assert!(mesh.tick());
  let res = <i64>::try_from(&result.0).unwrap();
  assert_eq!(res, 20i64);
}

#[test]
fn main() {
  initialize();

  let res = shards!(include_str!("test-simple.edn")).unwrap();
  let res = <i64>::try_from(&res.0).unwrap();
  assert_eq!(res, 99);
}

#[test]
fn edn_rs_usage() {
  initialize();

  let edn = edn!({:a 1 :b 2 :c 3});
  let edn = edn.to_string();
  assert_eq!(edn, "{:a 1, :b 2, :c 3, }");
  let res = shards!(edn).unwrap();
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
  shards_env!(root, "(def x 10)");
  let x = shards_env!(sub1, "(do (deflocal! y 11) x)").unwrap();
  let x = <i64>::try_from(&x.0).unwrap();
  assert_eq!(x, 10);
  let x = shards_env!(sub2, "(do (deflocal! y 12) x)").unwrap();
  let x = <i64>::try_from(&x.0).unwrap();
  assert_eq!(x, 10);
  let x = shards_env!(sub3, "(do (deflocal! y 13) x)").unwrap();
  let x = <i64>::try_from(&x.0).unwrap();
  assert_eq!(x, 10);
  let y = shards_env!(sub1, "y").unwrap();
  let y = <i64>::try_from(&y.0).unwrap();
  assert_eq!(y, 11);
  let y = shards_env!(sub2, "y").unwrap();
  let y = <i64>::try_from(&y.0).unwrap();
  assert_eq!(y, 12);
  let y = shards_env!(sub3, "y").unwrap();
  let y = <i64>::try_from(&y.0).unwrap();
  assert_eq!(y, 13);
}

#[test]
#[ignore]
fn fragment_get_data_test() {
  initialize();

  let mut hash: [u8; 32] = [0; 32];
  hex::decode_to_slice(
    "953f867f5e7af34b031d2689ea1486420571dfac0cd4043b173b0035e621c0dd",
    &mut hash,
  )
  .unwrap();

  let request = start_get_data(hash);
  let wire = <WireRef>::try_from(request.wire.0).unwrap();

  let mesh = Mesh::default();
  mesh.schedule(wire);

  loop {
    mesh.tick();
    let status = poll_chain(wire);
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
