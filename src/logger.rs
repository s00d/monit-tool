use chrono::Local;
use std::fs::File;
use std::io::{self, Write};

pub struct Logger {
    file: Option<File>,
}

impl Logger {
    pub fn new(logging: bool) -> io::Result<Self> {
        if logging {
            let now = Local::now();
            let file_name = format!("{}_{}.log", now.format("%Y-%m-%d"), now.format("%H-%M-%S"));
            let file = File::create(file_name)?;
            Ok(Logger { file: Some(file) })
        } else {
            Ok(Logger { file: None })
        }
    }

    pub fn log(&mut self, message: &str) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            writeln!(file, "{}", message)?;
        }
        Ok(())
    }
}
