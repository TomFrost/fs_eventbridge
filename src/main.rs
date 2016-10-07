/*
 * FS-EventBridge
 * Copyright (c) 2016, TechnologyAdvice LLC
 */

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate clap;

mod commands;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use std::process;
use regex::Regex;
use clap::{Arg, App};

// Traits
use std::io::Read;
use std::io::Write;
use std::error::Error;

// Global constants
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const ERR_UNKNOWN_CMD: &'static str = "ERR Unknown command. Send HELP for command list.";
const ERR_BAD_CMD_FORMAT: &'static str = "ERR Bad command format. Send HELP for details.";

macro_rules! eprintln {
    ($($tt:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stderr(), $($tt)*);
    }}
}

fn run_command(cmd: &str, args: &str) -> String {
    if cmd == "HELP" {
        commands::help::execute()
    } else if cmd == "CHANGE" {
        commands::change::execute(args)
    } else {
        String::from(ERR_UNKNOWN_CMD)
    }
}

fn process_line(line: &str) -> String {
    lazy_static! {
        static ref CMD_REGEX: Regex = Regex::new(r"^([A-Z]+)(?:\s(.+))?\s*").unwrap();
    }
    match CMD_REGEX.captures(line) {
        None => {
            String::from(ERR_BAD_CMD_FORMAT)
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
        };
        cmd_buf = {
            // Split command buffer into terminated lines
            let mut lines = cmd_buf.split(|b| *b == b'\n');
            // The last unterminated element is leftover buffer
            let new_buf = lines.next_back().unwrap().to_vec();
            for line in lines {
                // Convert to string, trim the CRs
                let line = String::from_utf8_lossy(line);
                let line = line.trim_matches('\r');
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

fn start(bind_ip: &str, port: &str) {
    let listen_on = format!("{}:{}", bind_ip, port);
    let listen_on = listen_on.as_str();
    let mut conn_count = 0;
    let listener = match TcpListener::bind(listen_on) {
        Err(e) => {
            eprintln!("Failed to start server: {}", e.description());
            process::exit(102);
        }
        Ok(m) => m
    };
    println!("FS-EventBridge v{} listening on {}", VERSION, listen_on);
    for stream in listener.incoming() {
        match stream {
            Err(e) => { eprintln!("failed: {}", e) }
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

fn main() {
    let matches = App::new("FS-EventBridge")
        .version(VERSION)
        .about("Filesystem event bridge server")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .help("The TCP port to open")
            .value_name("PORT")
            .default_value("65056")
            .empty_values(false))
        .arg(Arg::with_name("bind_ip")
            .short("i")
            .long("bind_ip")
            .help("The IP address of the interface on which to bind")
            .value_name("IP_ADDRESS")
            .default_value("0.0.0.0")
            .empty_values(false))
        .get_matches();

    start(matches.value_of("bind_ip").unwrap(), matches.value_of("port").unwrap());
}

#[cfg(test)]
mod tests {
    use super::run_command;
    use super::process_line;
    use super::ERR_UNKNOWN_CMD;
    use super::ERR_BAD_CMD_FORMAT;

    #[test]
    fn detects_unknown_command() {
        assert_eq!(run_command("null", "").as_str(), ERR_UNKNOWN_CMD);
    }

    #[test]
    fn detects_bad_command_format() {
        assert_eq!(process_line("foo bar").as_str(), ERR_BAD_CMD_FORMAT);
    }
}
