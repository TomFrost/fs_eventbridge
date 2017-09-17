/*
 * FS-EventBridge
 * Copyright (c) 2017 Tom Shawver LLC
 */

extern crate filetime;
extern crate time;
extern crate regex;

use self::regex::Regex;
use self::filetime::FileTime;

// Traits
use std::error::Error;

pub fn execute(args: &str) -> String {
    lazy_static! {
        static ref CHANGE_REGEX: Regex = Regex::new(r"^(.+?)(?:\s(\d+))?\s*$").unwrap();
    }
    match CHANGE_REGEX.captures(args) {
        None => String::from("ERR Invalid args"),
        Some(parsed_args) => {
            let path = parsed_args.at(1).unwrap();
            let mtime = parsed_args.at(2)
                .and_then(|m| m.parse().ok())
                .unwrap_or(time::get_time().sec as u64);
            let mtime_ft = FileTime::from_seconds_since_1970(mtime, 0);
            match filetime::set_file_times(path, mtime_ft, mtime_ft) {
                Err(e) => format!("ERR {}", e.description()),
                Ok(_) => format!("OK {}", args)
            }
        }
    }
}
