// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

//Imports
use rand::{thread_rng, Rng};
use std::fs::{metadata, File};
use std::io::{Read, Seek, SeekFrom, Write};

// Function to perform wrapping addition of two u8 values
fn wrapping_add(a: u8, b: u8) -> u8 {
    let sum = a as u16 + b as u16; // add the two u8 values as u16
    if sum > u8::max as u16 {
        // check if the sum exceeds the maximum value for u8
        (sum - u8::max as u16 - 1) as u8 // perform wrapping if sum exceeds u8 maximum value
    } else {
        sum as u8 // return the sum as u8 if it doesn't exceed maximum value
    }
}

/// Performs wrapping subtraction for u8 values
fn wrapping_subtract(a: u8, b: u8) -> u8 {
    if a >= b {
        a - b // If a is greater than or equal to b, perform normal subtraction
    } else {
        // If a is less than b, perform wrapping subtraction
        // Since a < b, a - b would underflow, so add u8::max to a and add 1 to compensate
        (u16::from(a) + u16::from(u8::max_value()) + 1 - u16::from(b)) as u8
    }
}

// Function to get file size
fn file_size(path: &str) -> Result<u64, std::io::Error> {
    let metadata = metadata(path)?; // Get metadata of the file
    Ok(metadata.len()) // Return the size of the file
}

#[tauri::command]
fn encode(path: &str, message: &str) -> String {
    const OFFSET_VALUE_ADDRESS: u64 = 0x100; // Define address for offset value in the file
    const LETTER_SHIFT: u8 = 128; // Define shift value for encoding letters
    const MAX_FILE_SIZE: u64 = 1000; // Define maximum file size

    let file_size = match file_size(&path) {
        Ok(size) => size,
        Err(err) => return format!("Error: {:?}", err), // Return error as string
    };

    if file_size < MAX_FILE_SIZE {
        // Check if the file size is less than the maximum allowed
        return format!("Error: File has to be over 10kB"); // Return an error if the file size is too small
    }

    let mut file = match File::options().write(true).open(path) {
        Ok(file) => file,
        Err(err) => return format!("Error: {:?}", err), // Return error as string
    };

    let message_bytes: &[u8] = message.as_bytes(); // Get bytes of the message

    let offset_value = thread_rng().gen_range(5..=15); // Generate a random offset value

    let message_length_bytes: u8 = match message.len().try_into() {
        Ok(length) => length,
        Err(_) => return format!("Message has to be max 255 bytes long"), // Return error as string
    };

    if let Err(err) = file.seek(SeekFrom::Start(OFFSET_VALUE_ADDRESS)) {
        return format!("Error: {:?}", err); // Return error as string
    }

    // Write offset value and message length to the file
    if let Err(err) = file.write(&[offset_value]) {
        return format!("Error: {:?}", err); // Return error as string
    }

    if let Err(err) = file.write(&[message_length_bytes]) {
        return format!("Error: {:?}", err); // Return error as string
    }

    if let Err(err) = file.seek(SeekFrom::Current(-1)) {
        return format!("Error: {:?}", err); // Return error as string
    }

    for byte in message_bytes {
        // Iterate over each byte in the message
        let pointer_location = match file.seek(SeekFrom::Current(offset_value as i64 - 1)) {
            Ok(location) => location,
            Err(err) => return format!("Error: {:?}", err), // Return error as string
        };

        if pointer_location > file_size {
            return format!("Error: End of line detected"); // Return error as string
        }

        let shifted_byte: u8 = wrapping_add(*byte, LETTER_SHIFT); // Perform wrapping addition for each byte

        if let Err(err) = file.write(&[shifted_byte]) {
            return format!("Error: {:?}", err); // Return error as string
        }
    }

    "Successfully Encoded Message!".to_string() // Return Ok if encoding is successful
}

/// Decodes a message from a file
#[tauri::command]
fn decode(path: &str) -> String {
    // Constants
    const OFFSET_VALUE_ADDRESS: u64 = 0x100;
    const LETTER_SHIFT: u8 = 128;

    // Read offset value and message length bytes from file
    let mut offset_value: [u8; 1] = [0];
    let mut message_length_bytes: [u8; 1] = [0];
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => return format!("Error: {:?}", err), // Return error as string
    };

    // Read offset value and message length bytes
    if let Err(err) = file.seek(SeekFrom::Start(OFFSET_VALUE_ADDRESS)) {
        return format!("Error: {:?}", err); // Return error as string
    }

    if let Err(err) = file.read(&mut offset_value) {
        return format!("Error: {:?}", err); // Return error as string
    }

    if let Err(err) = file.read(&mut message_length_bytes) {
        return format!("Error: {:?}", err); // Return error as string
    }

    // Reset file pointer to offset value address
    if let Err(err) = file.seek(SeekFrom::Start(OFFSET_VALUE_ADDRESS)) {
        return format!("Error: {:?}", err); // Return error as string
    }

    // Decode message
    let mut message_bytes: Vec<u8> = Vec::new();
    for _ in 0..message_length_bytes[0] {
        if let Err(err) = file.seek(SeekFrom::Current(offset_value[0] as i64)) {
            return format!("Error: {:?}", err); // Return error as string
        }

        let mut message_buffer: [u8; 1] = [0];
        if let Err(err) = file.read(&mut message_buffer) {
            return format!("Error: {:?}", err); // Return error as string
        }

        if let Err(err) = file.seek(SeekFrom::Current(-1)) {
            return format!("Error: {:?}", err); // Return error as string
        }

        message_bytes.push(wrapping_subtract(message_buffer[0], LETTER_SHIFT)); // Perform wrapping subtraction
    }

    // Convert message bytes to string
    let message_string = match String::from_utf8(message_bytes) {
        Ok(message_string) => message_string,
        Err(_) => return "Error: Invalid UTF-8 sequence".to_string(), // Return error as string
    };

    message_string
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![encode, decode])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
