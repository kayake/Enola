use std::sync::Mutex;
use std::io::{self, Write}
use chrono::Local;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Info = 0,
    Warn,
    Error,
    Debug,
    Found,
    NotFound,
    Request,
    Response
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "\x1b[32mINF\x1b[0m",
            LogLevel::Warn => "\x1b[33mWARN\x1b[0m",
            LogLevel::Error => "\x1b[31mERR\x1b[0m",
            LogLevel::Debug => "\x1b[34mDBG\x1b[0m",
            LogLevel::Found => "\x1b[92mFOUND\x1b[0m",
            LogLevel::NotFound => "\x1b[91m!FOUND\x1b[0m",
            LogLevel::Request => "\x1b[36mREQ\x1b[0m",
            LogLevel::Response => "\x1b[95mRES\x1b[0m"
        }
    }
}


struct Logger {
    level: LogLevel,
    output: Mutex<Box<dyn Write + Send>>,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Logger {
            level,
            output: Mutex::new(Box::new(io::stdout()))
        }
    }

    pub fn log(&self, level: LogLevel, message: &str, bold: bool) {
        if level >= self.level {
            let mut out = self.output.lock().unwrap();
            let key = if bold {
                format!("\x1b[1m{}\x1b[0m", level.as_str())
            } else {
                level.as_str().to_string()
            }
            writeln!(out, "[ {} ] {}", key, msg).unwrap();
        }
    }


    pub fn inf(&self, message: &str, bold: bool) { self.log(LogLevel::Info, message, bold) } 
    pub fn warn(&self, message: &str, bold: bool) { self.log(LogLevel::Warn, message, bold) }
    pub fn err(&self, message: &str, bold: bool) { self.log(LogLevel::Error, message, bold) }
    pub fn dbg(&self, message: &str, bold: bool) { self.log(LogLevel::Debug, message, bold) }
    pub fn fnd(&self, message: &str, bold: bool) { self.log(LogLevel::Found, message, bold) }
    pub fn nfnd(&self, message: &str, bold: bool) { self.log(LogLevel::NotFound, message, bold) }
    pub fn req(&self, message: &str, bold: bool) { self.log(LogLevel::Request, message, bold) }
    pub fn res(&self, message: &str, bold: bool) { self.log(LogLevel::Response, message, bold) }
}
