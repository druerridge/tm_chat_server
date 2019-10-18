use std::fs;

use structopt::StructOpt;

use crate::chat_server;
use crate::settings::Settings;

pub struct App {}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short = "c", long = "config", help = "Set config file location", default_value = "C:\\workspace\\rust\\tm_chat_server\\config\\settings.json")]
    config_file: String,
}

impl App {
    pub fn run(self) {
        println!("Begun");

        let args: Args = Args::from_args();
        let settings_path = args.config_file;
        println!("Reading config from {0}", &settings_path);
        let json_str = fs::read_to_string(&settings_path)
            .unwrap_or_else(|_| panic!("Unable to read from file at {0}", &settings_path));
        let settings: Settings = serde_json::from_str(json_str.as_str())
            .unwrap_or_else(|_| panic!("Unable to parse malformed json:\n {0}", json_str));
        println!("port {0}", settings.port);

        chat_server::run(settings.port);

        println!("Finished");
    }
}