use chainblocks::types::Table;
use clap::{App, AppSettings, Arg};
use claylib::proto_upload;
use std::io::Read;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
  INIT.call_once(|| {
    chainblocks::core::init();
  });
}

fn main() {
  initialize();
  let matches = App::new("claytool")
    .about("claymore utility")
    .version("0.1")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .author("Fragcolor Pte. Ltd.")
    .subcommand(
      App::new("proto")
        .short_flag('P')
        .long_flag("proto")
        .about("Proto-fragments management tools.")
        .arg(
          Arg::new("file")
            .short('u')
            .long("upload")
            .help("Uploads a proto-fragment to a Clamor node")
            .takes_value(true)
            .required_unless_present("help"),
        )
        .arg(
          Arg::new("type")
            .short('t')
            .long("type")
            .help("The type of the proto-fragment to upload")
            .takes_value(true)
            .required_unless_present("help"),
        )
        .arg(
          Arg::new("container")
            .short('c')
            .long("container")
            .help("The container type of the audio proto-fragment to upload")
            .takes_value(true)
            .required_if_eq("type", "audio"),
        )
        .arg(
          Arg::new("node")
            .short('n')
            .long("node")
            .help("The Clamor node endpoint to send commands to")
            .default_value("http://127.0.0.1:9933")
            .takes_value(true),
        )
        .arg(
          Arg::new("signer")
            .short('s')
            .long("signer")
            .help("The private key (or mnemonic/preset) of the signer account")
            .default_value("//Dave")
            .takes_value(true),
        ),
    )
    .get_matches();

  match matches.subcommand() {
    Some(("proto", matches)) => {
      if let Some(upload) = matches.value_of("file") {
        let node = matches.value_of("node").unwrap();
        let type_ = matches.value_of("type").unwrap();
        let signer = matches.value_of("signer").unwrap();
        let mut file = std::fs::File::open(upload).expect("File to upload as proto-fragment");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        let container = matches.value_of("container").unwrap();

        match container {
          "ogg" | "mp3" => {}
          _ => panic!("Invalid container type"),
        }

        let mut table = Table::new();
        table.insert_fast_static("container\0", type_.into());
        table.insert_fast_static("data\0", buffer[..].into());

        proto_upload(node, signer, type_, table).unwrap();
      }
    }
    _ => {}
  }
}
