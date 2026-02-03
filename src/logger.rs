use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Log seviyeleri (sade ama genişletilebilir)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Basit ama ayarlanabilir logger
pub struct Logger {
    min_level: LogLevel,
    to_stdout: bool,
    file: Option<File>,
}

impl Logger {
    /// Dosyaya log yazan logger üretir
    pub fn new(file_path: &str) -> Self {
        let file = Self::open_log_file(file_path);
        Self {
            min_level: LogLevel::Info,
            to_stdout: false,
            file,
        }
    }

    /// Seviye filtreleme (min seviyeden aşağısı yazılmaz)
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    /// Stdout yazımını aç/kapat
    pub fn set_stdout(&mut self, enabled: bool) {
        self.to_stdout = enabled;
    }

    /// Tek satır log yaz
    pub fn log(&mut self, level: LogLevel, message: &str) {
        if level < self.min_level {
            return;
        }

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let line = format!("[{}] {:?} | {}\n", ts, level, message);

        if self.to_stdout {
            print!("{}", line);
        }

        if let Some(file) = &mut self.file {
            let _ = file.write_all(line.as_bytes());
        }
    }

    /// Çoklu satır log yazmak için yardımcı
    pub fn log_many(&mut self, level: LogLevel, lines: &[String]) {
        for line in lines {
            self.log(level, line);
        }
    }

    fn open_log_file(file_path: &str) -> Option<File> {
        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
    }
}
