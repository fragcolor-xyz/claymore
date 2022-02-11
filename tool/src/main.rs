use clap::{App, AppSettings, Arg};
use std::io::Read;
use claymore::proto_upload;

fn main() {
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
          Arg::new("node")
            .short('n')
            .long("node")
            .help("The Clamor node endpoint to send commands to")
            .default_value("http://127.0.0.1:9933")
            .takes_value(true),
        ),
    )
    .get_matches();

  match matches.subcommand() {
    Some(("proto", matches)) => {
      if let Some(upload) = matches.value_of("file") {
        let node = matches.value_of("node").unwrap();
        let type_ = matches.value_of("type").unwrap();
        let mut file = std::fs::File::open(upload).expect("File to upload as proto-fragment");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        proto_upload(node, type_, &buffer).unwrap();
      }
    }
    _ => {}
  }
}
