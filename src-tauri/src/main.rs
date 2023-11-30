// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![client])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


use futures::future::join_all;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main(flavor = "current_thread")]

#[tauri::command]
async fn client() -> Result<(), Box<dyn std::error::Error>> {
    use tokio_modbus::prelude::*;

    let socket_addr = "127.0.0.1:502".parse().unwrap();

    let mut ctx = Arc::new(Mutex::new(tcp::connect_slave(socket_addr, Slave(1)).await?));

    let mut handles = vec![];

    for i in 0..10 {
        let clone = Arc::clone(&ctx);
        let handle = tokio::spawn(async move{
            // let data = clone.lock().await.write_multiple_registers(10, & [55,66,77,88]).await;
            let data = clone.lock().await.read_holding_registers(10, 10).await;
            println!("{:?}", data);
        });
        handles.push(handle);
    }
    join_all(handles).await;


    Ok(())
}