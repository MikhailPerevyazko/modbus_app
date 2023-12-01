// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use clap:: {
    Args,
    Parser
};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
// use tauri::Manager;
// use std::fs;
use serde_json;
use serde_yaml;
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

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
        .manage(PathBuf(file_path))
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    let timeout = Duration::from_secs(1);

    // open TCP connection
    let mut stream = TcpStream::connect("127.0.0.1:5500").unwrap();
    stream.set_read_timeout(Some(timeout)).unwrap();
    stream.set_write_timeout(Some(timeout)).unwrap();

    // create request object
    let mut mreq = ModbusRequest::new(1, ModbusProto::TcpUdp);
    mreq.tr_id = 2; // just for test, default tr_id is 1

    // set 2 coils
    let mut request = Vec::new();
    mreq.generate_set_coils_bulk(8, &[true, true], &mut request)
        .unwrap();

    // write request to stream
    stream.write(&request).unwrap();

    // read first 6 bytes of response frame
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    // read rest of response frame
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }
    // check if frame has no Modbus error inside
    mreq.parse_ok(&response).unwrap();

    // get coil values back
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
    // check if frame has no Modbus error inside and parse response bools into data vec
    mreq.parse_bool(&response, &mut data).unwrap();
    for i in 0..data.len() {
        println!("{} {}", i, data[i]);
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct  ModbusItems {
    pub host: String,
    pub port: i32,
    pub pause: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VariablesItems {
    pub var_name: String,
    pub storage_type: String,
}
