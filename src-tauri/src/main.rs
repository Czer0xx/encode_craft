// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use std::fs::File;
use std::io::{prelude::*, ErrorKind, SeekFrom};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn encode(path: String, message: String) -> String {
    const STARTING_POINT: [u8; 3] = [0xFF, 0x11, 0xFF];

    let file_name = path;
    let message_bytes = message.as_bytes();

    let mut file = File::options()
        .read(true)
        .write(true)
        .open(file_name.trim())
        .unwrap();

    let mut zero_byte_adress: u64 = 0;
    loop {
        let mut buffer: [u8; 1] = [0];
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(_) => break,
        };
        if buffer[0] == 0 {
            match file.stream_position() {
                Ok(position) => {
                    zero_byte_adress = position - 1;
                    if zero_byte_adress >= 0x1C5 {
                        break;
                    }
                }
                Err(e) => panic!("{}", e),
            };
        }
    }

    file.seek(SeekFrom::Start(zero_byte_adress)).unwrap();
    match file.write_all(&STARTING_POINT) {
        Ok(_) => {
            let hex_string = format!("{:x}", zero_byte_adress);
            println!(
                "Succesfully wrote STARTING_POINT starting from adress: 0x{}",
                hex_string
            );
        }
        Err(e) => panic!("{:?}", e),
    }
    file.write_all(message_bytes).unwrap();
    match file.write_all(&STARTING_POINT) {
        Ok(_) => {
            let hex_string = format!("{:x}", file.stream_position().unwrap());
            println!(
                "Succesfully wrote ENDING_POINT ending on adress: 0x{}",
                hex_string
            );
        }
        Err(e) => panic!("{:?}", e),
    }
    let starting_point_hex: String = format!("{:x}", zero_byte_adress);
    let ending_point_hex: String = format!("{:x}", file.stream_position().unwrap());
    let result_string = format!("{},{}", starting_point_hex, ending_point_hex);
    result_string
}

#[tauri::command]
fn decode(path: String) -> String {
    let mut file = File::open(path).unwrap();

    let mut message_bytes: Vec<u8> = Vec::new();

    loop {
        let mut buffer: [u8; 1] = [0];
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return ("Nie znaleziono zakodowanej wiadomosci!").to_string();
                }
            }
        };
        if buffer[0] == 0xFF {
            file.read_exact(&mut buffer).unwrap();
            if buffer[0] == 0x11 {
                file.read_exact(&mut buffer).unwrap();
                if buffer[0] == 0xFF {
                    let mut message_buffer: [u8; 1] = [0];
                    while message_buffer[0] != 0xFF {
                        file.read_exact(&mut message_buffer).unwrap();
                        if message_buffer[0] != 0xFF {
                            message_bytes.push(message_buffer[0]);
                        }
                    }
                    break;
                }
            }
        };
    }
    let message_string = String::from_utf8_lossy(&message_bytes).into_owned();
    message_string
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, encode, decode])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
