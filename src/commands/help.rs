/*
 * FS-EventBridge
 * Copyright (c) 2016, TechnologyAdvice LLC
 */

pub fn execute() -> String {
    String::from("Commands:\n\
        CHANGE /path/to/file mtime\n\
        \tMarks the given file path as changed. The mtime argument can optionally\n\
        \tbe specified (in seconds) to set an explicit modified time.")
}
