// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use serde_json;
use serde_yaml;
use clap::Parser;


fn main() {
    let args = Args::parse();
    let home_dir = std::env::var("HOME").unwrap();
    let file_path = match args.file_path {
        Some(path) => path,
        None => PathBuf::from(home_dir)
            .join(".config")
            .join("simple_modbusclient")
            .join("config.yaml"),
    };
    tauri::Builder::default()
        .manage(PathState(file_path))
        .invoke_handler(tauri::generate_handler![config_serialize])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    let timeout = Duration::from_secs(1);

    let mut stream = TcpStream::connect("127.0.0.1:5500").unwrap();
    stream.set_read_timeout(Some(timeout)).unwrap();
    stream.set_write_timeout(Some(timeout)).unwrap();

    let mut mreq = ModbusRequest::new(1, ModbusProto::TcpUdp);
    mreq.tr_id = 2;

    let mut request = Vec::new();
    mreq.generate_set_coils_bulk(8, &[true, true], &mut request)
        .unwrap();

    stream.write(&request).unwrap();

    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }

    mreq.parse_ok(&response).unwrap();

    mreq.generate_get_coils(0, 5, &mut request).unwrap();
    stream.write(&request).unwrap();
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }
    let mut data = Vec::new();
    mreq.parse_bool(&response, &mut data).unwrap();
    for i in 0..data.len() {
        println!("{} {}", i, data[i]);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigItems {
    host: String,
    port: i64,
    pause: i64,
    var_name: String,
    storage_type: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    config: Vec<ConfigItems>,
}
struct PathState(PathBuf);
impl PathState {
    fn path(&self) -> PathBuf {
        self.0.to_owned()
    }
}

#[tauri::command]
fn config_serialize(state: tauri::State<PathState>) -> String {
    let file = std::fs::File::open(state.path()).unwrap();
    let json_config: Config = serde_yaml::from_reader(file).unwrap();
    let config = serde_json::to_string_pretty(&json_config).unwrap();
    config
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_path: Option<PathBuf>,
}