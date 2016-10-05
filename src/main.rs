/*
 * FS-EventBridge
 * Copyright (c) 2016, TechnologyAdvice LLC
 */

#[macro_use] extern crate lazy_static;
extern crate regex;

mod commands;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use regex::Regex;

// Traits
use std::io::Read;
use std::io::Write;

// Global constants
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn run_command(cmd: &str, args: &str) -> String {
    if cmd == "HELP" {
        commands::help::execute()
    } else if cmd == "CHANGE" {
        commands::change::execute(args)
    } else {
        String::from("ERR Unknown command. Send HELP for command list.")
    }
}

fn process_line(line: &str) -> String {
    lazy_static! {
        static ref CMD_REGEX: Regex = Regex::new(r"^([A-Z]+)(?:\s(.+))?\s*").unwrap();
    }
    match CMD_REGEX.captures(line) {
        None => {
            String::from("ERR Bad command format. Send HELP for details.")
        },
        Some(cmd_caps) => {
            let cmd = cmd_caps.at(1).unwrap();
            let args = cmd_caps.at(2).unwrap_or("");
            run_command(cmd, args)
        }
    }
}

fn handle_client(client_num: u32, mut stream: TcpStream) {
    let mut rx_buf = [0; 512];
    let mut cmd_buf = Vec::with_capacity(512);

    println!("[{}] Client connected", client_num);

    'mainloop: loop {
        // Read incoming data
        match stream.read(&mut rx_buf) {
            Err(e) => panic!("[{}] Got an error: {}", client_num, e),
            Ok(0) => {
                println!("[{}] Client disconnected", client_num);
                break;
            },
            Ok(n) => cmd_buf.extend_from_slice(&rx_buf[..n])
        }
        cmd_buf = {
            // Split command buffer into terminated lines
            let mut lines = rx_buf.split(|b| *b == b'\n');
            // The last unterminated element is leftover buffer
            let new_buf = lines.next_back().unwrap().to_vec();
            for line in lines {
                // Convert to string, trim the CRs
                let line = String::from_utf8_lossy(line);
                let line = line.trim_right_matches('\r');
                println!("[{}|RX] {}", client_num, line);
                let mut resp = process_line(line);
                println!("[{}|TX] {}", client_num, resp);
                resp.push('\n');
                if stream.write(resp.as_bytes()).is_err() {
                    break 'mainloop;
                }
            }
            new_buf
        }
    }
}

fn main() {
    let listen_on = "0.0.0.0:65056";
    let mut conn_count = 0;
    let listener = TcpListener::bind(listen_on).unwrap();
    println!("FS-EventBridge v{} listening on {}", VERSION, listen_on);
    for stream in listener.incoming() {
        match stream {
            Err(e) => { println!("failed: {}", e) }
            Ok(stream) => {
                let client_num: u32 = conn_count;
                conn_count = conn_count.wrapping_add(1);
                thread::spawn(move || {
                    handle_client(client_num, stream)
                });
            }
        }
    }
}

