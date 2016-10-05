/*
 * FS-EventBridge
 * Copyright (c) 2016, TechnologyAdvice LLC
 */

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate filetime;
extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use regex::Regex;
use filetime::FileTime;

// Traits
use std::io::Read;
use std::io::Write;
use std::error::Error;

// Global constants
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn run_command(cmd: &str, args: &str) -> String {
    lazy_static! {
        static ref CHANGE_REGEX: Regex = Regex::new(r"^(.+?)(?:\s(\d+))?\s*$").unwrap();
    }
    let mut resp = String::new();
    if cmd == "HELP" {
        resp.push_str("Commands:\n\
            CHANGE /path/to/file mtime\n\
            \tMarks the given file path as changed. The mtime argument can optionally\n\
            \tbe specified (in seconds) to set an explicit modified time.\n");
    } else if cmd == "CHANGE" {
        let parsed_args_opts = CHANGE_REGEX.captures(args);
        let mut path = "";
        let mut mtime: u64 = 0;
        match parsed_args_opts {
            None => resp.push_str("ERR Invalid args"),
            Some(_) => {
                let parsed_args = parsed_args_opts.unwrap();
                path = parsed_args.at(1).unwrap();
                match parsed_args.at(2) {
                    None => mtime = time::get_time().sec as u64,
                    Some(m) => mtime = m.parse().unwrap(),
                }
            }
        }
        if resp.len() == 0 {
            let mtime_ft = FileTime::from_seconds_since_1970(mtime, 0);
            match filetime::set_file_times(path, mtime_ft, mtime_ft) {
                Err(e) => {
                    resp.push_str("ERR ");
                    resp.push_str(e.description());
                },
                Ok(_) => {
                    resp = String::from("OK ");
                    resp.push_str(args);
                }
            }
        }
    } else {
        resp.push_str("ERR Unknown command. Send HELP for command list.");
    }
    resp
}

fn process_line(line: &str) -> String {
    lazy_static! {
        static ref CMD_REGEX: Regex = Regex::new(r"^([A-Z]+)(?:\s(.+))?\s*").unwrap();
    }
    let resp: String;
    let cmd_caps_opt = CMD_REGEX.captures(line);
    match cmd_caps_opt {
        None => {
            resp = String::from("ERR Bad command format. Send HELP for details.");
        },
        Some(_) => {
            let cmd_caps = cmd_caps_opt.unwrap();
            let cmd = cmd_caps.at(1).unwrap();
            let args = cmd_caps.at(2).unwrap_or("");
            resp = run_command(&cmd, &args);
        },
    };
    resp
}

fn handle_client(client_num: u32, mut stream: TcpStream) {
    println!("[{}] Client connected", client_num);
    lazy_static! {
        static ref CRLF_REGEX: Regex = Regex::new(r"[\r\n]+").unwrap();
    }
    let mut byte_buf;
    let mut str_buf = String::from("");
    let mut failed = false;
    loop {
        // Start with a 0-filled byte buffer
        byte_buf = [0; 512];
        let _ = match stream.read(&mut byte_buf) {
            Err(e) => panic!("[{}] Got an error: {}", client_num, e),
            Ok(m) => {
                if m == 0 {
                    // EOF
                    println!("[{}] Client disconnected", client_num);
                    break;
                }
                m
            },
        };
        // Parse the string data from the byte buffer, crushing line breaks to a single \n
        let input = str::from_utf8(&byte_buf).unwrap().trim_right_matches('\u{0}');
        let input = CRLF_REGEX.replace(input, "\n");
        let input = input.as_str();
        // Append the new string data after any unprocessed data we already have
        str_buf.push_str(input);
        // Split our lines at the \n. The last element (usually "") becomes unprocessed data
        let str_buf_clone = str_buf.clone();
        let mut lines: Vec<&str> = str_buf_clone.split('\n').collect();
        str_buf = String::from(lines.pop().unwrap_or(""));
        // Iterate over all \n-terminated lines
        let line_iterator = lines.iter();
        for line in line_iterator {
            println!("[{}|RX] {}", client_num, line);
            let mut resp = process_line(line);
            println!("[{}|TX] {}", client_num, resp);
            resp.push('\n');
            let bytes = resp.as_bytes();
            match stream.write(&bytes) {
                Err(_) => {
                    failed = true;
                    break;
                },
                Ok(_) => continue,
            };
        }
        if failed {
            break;
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
                let client_num:u32 = conn_count;
                conn_count = (conn_count + 1) % std::u32::MAX;
                thread::spawn(move || {
                    handle_client(client_num, stream)
                });
            }
        }
    }
}

