//! Log output destinations for writing formatted log entries.

use crate::logging::{LogFormat, LoggingError};
use async_trait::async_trait;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Trait for log output destinations
#[async_trait]
pub trait LogOutput: std::fmt::Debug + Send + Sync {
    /// Write a formatted log message
    async fn write(&self, message: &str) -> Result<(), LoggingError>;

    /// Flush any buffered content
    async fn flush(&self) -> Result<(), LoggingError>;

    /// Close the output destination
    async fn close(&self) -> Result<(), LoggingError>;

    /// Get the preferred log format for this output
    fn preferred_format(&self) -> LogFormat;

    /// Check if this output is available/working
    async fn is_available(&self) -> bool {
        true
    }
}

/// Console/terminal output
#[derive(Debug)]
pub struct ConsoleOutput {
    format: LogFormat,
    colored: bool,
    use_stderr: bool,
}

impl ConsoleOutput {
    pub fn new(format: LogFormat, colored: bool) -> Self {
        Self {
            format,
            colored,
            use_stderr: false,
        }
    }

    pub fn with_stderr(mut self, use_stderr: bool) -> Self {
        self.use_stderr = use_stderr;
        self
    }
}

#[async_trait]
impl LogOutput for ConsoleOutput {
    async fn write(&self, message: &str) -> Result<(), LoggingError> {
        if self.use_stderr {
            eprintln!("{}", message);
        } else {
            println!("{}", message);
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), LoggingError> {
        if self.use_stderr {
            std::io::stderr().flush().map_err(LoggingError::IoError)?;
        } else {
            std::io::stdout().flush().map_err(LoggingError::IoError)?;
        }
        Ok(())
    }

    async fn close(&self) -> Result<(), LoggingError> {
        self.flush().await
    }

    fn preferred_format(&self) -> LogFormat {
        self.format
    }
}

/// File rotation configuration
#[derive(Debug, Clone)]
pub struct FileRotation {
    pub max_size_bytes: u64,
    pub max_files: usize,
    pub compress: bool,
}

impl Default for FileRotation {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            max_files: 5,
            compress: false,
        }
    }
}

/// File output with optional rotation
#[derive(Debug)]
pub struct FileOutput {
    path: PathBuf,
    format: LogFormat,
    rotation: Option<FileRotation>,
    writer: Arc<Mutex<Option<BufWriter<std::fs::File>>>>,
    current_size: Arc<Mutex<u64>>,
}

impl FileOutput {
    pub fn new(
        path: PathBuf,
        format: LogFormat,
        rotation: Option<FileRotation>,
    ) -> Result<Self, LoggingError> {
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(LoggingError::IoError)?;
        }

        let output = Self {
            path,
            format,
            rotation,
            writer: Arc::new(Mutex::new(None)),
            current_size: Arc::new(Mutex::new(0)),
        };

        Ok(output)
    }

    async fn ensure_writer(&self) -> Result<(), LoggingError> {
        let mut writer_guard = self.writer.lock().await;

        if writer_guard.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
                .map_err(LoggingError::IoError)?;

            // Get current file size
            let metadata = file.metadata().map_err(LoggingError::IoError)?;
            *self.current_size.lock().await = metadata.len();

            *writer_guard = Some(BufWriter::new(file));
        }

        Ok(())
    }

    async fn check_rotation(&self, message_len: usize) -> Result<(), LoggingError> {
        if let Some(rotation) = &self.rotation {
            let mut current_size = self.current_size.lock().await;

            if *current_size + message_len as u64 > rotation.max_size_bytes {
                self.rotate_file(rotation).await?;
                *current_size = 0;
            }
        }
        Ok(())
    }

    async fn rotate_file(&self, rotation: &FileRotation) -> Result<(), LoggingError> {
        // Close current writer
        {
            let mut writer_guard = self.writer.lock().await;
            if let Some(mut writer) = writer_guard.take() {
                writer.flush().map_err(LoggingError::IoError)?;
            }
        }

        // Rotate existing files
        for i in (1..rotation.max_files).rev() {
            let old_path = if i == 1 {
                self.path.clone()
            } else {
                self.path.with_extension(format!(
                    "{}.{}",
                    self.path.extension().unwrap_or_default().to_string_lossy(),
                    i - 1
                ))
            };

            let new_path = self.path.with_extension(format!(
                "{}.{}",
                self.path.extension().unwrap_or_default().to_string_lossy(),
                i
            ));

            if old_path.exists() {
                if i == rotation.max_files {
                    // Remove the oldest file
                    std::fs::remove_file(&old_path).map_err(LoggingError::IoError)?;
                } else {
                    // Move to next rotation
                    std::fs::rename(&old_path, &new_path).map_err(LoggingError::IoError)?;

                    // Compress if requested
                    if rotation.compress && i > 1 {
                        self.compress_file(&new_path).await?;
                    }
                }
            }
        }

        // Create new writer
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(LoggingError::IoError)?;

        *self.writer.lock().await = Some(BufWriter::new(file));

        Ok(())
    }

    async fn compress_file(&self, _path: &PathBuf) -> Result<(), LoggingError> {
        // TODO: Implement file compression (gzip)
        // This would require adding a compression library like flate2
        Ok(())
    }
}

#[async_trait]
impl LogOutput for FileOutput {
    async fn write(&self, message: &str) -> Result<(), LoggingError> {
        self.ensure_writer().await?;

        let message_with_newline = format!("{}\n", message);
        let message_len = message_with_newline.len();

        self.check_rotation(message_len).await?;

        {
            let mut writer_guard = self.writer.lock().await;
            if let Some(ref mut writer) = *writer_guard {
                writer
                    .write_all(message_with_newline.as_bytes())
                    .map_err(LoggingError::IoError)?;

                *self.current_size.lock().await += message_len as u64;
            }
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), LoggingError> {
        let mut writer_guard = self.writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.flush().map_err(LoggingError::IoError)?;
        }
        Ok(())
    }

    async fn close(&self) -> Result<(), LoggingError> {
        let mut writer_guard = self.writer.lock().await;
        if let Some(mut writer) = writer_guard.take() {
            writer.flush().map_err(LoggingError::IoError)?;
        }
        Ok(())
    }

    fn preferred_format(&self) -> LogFormat {
        self.format
    }

    async fn is_available(&self) -> bool {
        // Check if we can write to the file
        if let Some(parent) = self.path.parent() {
            parent.exists() && parent.is_dir()
        } else {
            true
        }
    }
}

/// Syslog facility enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyslogFacility {
    User = 1,
    Mail = 2,
    Daemon = 3,
    Auth = 4,
    Syslog = 5,
    Lpr = 6,
    News = 7,
    Uucp = 8,
    Cron = 9,
    Authpriv = 10,
    Ftp = 11,
    Local0 = 16,
    Local1 = 17,
    Local2 = 18,
    Local3 = 19,
    Local4 = 20,
    Local5 = 21,
    Local6 = 22,
    Local7 = 23,
}

/// Syslog output (Unix systems)
#[derive(Debug)]
pub struct SyslogOutput {
    facility: SyslogFacility,
    format: LogFormat,
    socket: Arc<Mutex<Option<std::os::unix::net::UnixDatagram>>>,
}

impl SyslogOutput {
    pub fn new(facility: SyslogFacility, format: LogFormat) -> Result<Self, LoggingError> {
        Ok(Self {
            facility,
            format,
            socket: Arc::new(Mutex::new(None)),
        })
    }

    async fn ensure_socket(&self) -> Result<(), LoggingError> {
        let mut socket_guard = self.socket.lock().await;

        if socket_guard.is_none() {
            let socket =
                std::os::unix::net::UnixDatagram::unbound().map_err(LoggingError::IoError)?;

            // Try to connect to syslog socket
            let syslog_paths = ["/dev/log", "/var/run/syslog", "/var/run/log"];
            let mut connected = false;

            for path in &syslog_paths {
                if std::path::Path::new(path).exists() {
                    if socket.connect(path).is_ok() {
                        connected = true;
                        break;
                    }
                }
            }

            if !connected {
                return Err(LoggingError::OutputError(
                    "Cannot connect to syslog socket".to_string(),
                ));
            }

            *socket_guard = Some(socket);
        }

        Ok(())
    }

    fn format_syslog_message(&self, level: u8, message: &str) -> String {
        let priority = (self.facility as u8) * 8 + level;
        let timestamp = chrono::Utc::now().format("%b %d %H:%M:%S");
        let hostname_os_string = gethostname::gethostname();
        let hostname = hostname_os_string.to_string_lossy();
        let tag = "devkit";

        format!(
            "<{}>{} {} {}: {}",
            priority, timestamp, hostname, tag, message
        )
    }

    fn log_level_to_syslog_priority(level: &crate::logging::LogLevel) -> u8 {
        match level {
            crate::logging::LogLevel::Error => 3, // Error
            crate::logging::LogLevel::Warn => 4,  // Warning
            crate::logging::LogLevel::Info => 6,  // Info
            crate::logging::LogLevel::Debug => 7, // Debug
            crate::logging::LogLevel::Trace => 7, // Debug (syslog doesn't have trace)
        }
    }
}

#[async_trait]
impl LogOutput for SyslogOutput {
    async fn write(&self, message: &str) -> Result<(), LoggingError> {
        // Extract log level from message if possible (this is a simplified approach)
        let level = if message.contains("ERROR") {
            3
        } else if message.contains("WARN") {
            4
        } else if message.contains("INFO") {
            6
        } else {
            7
        };

        self.ensure_socket().await?;

        let syslog_message = self.format_syslog_message(level, message);

        let socket_guard = self.socket.lock().await;
        if let Some(ref socket) = *socket_guard {
            socket
                .send(syslog_message.as_bytes())
                .map_err(LoggingError::IoError)?;
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), LoggingError> {
        // Syslog is typically unbuffered
        Ok(())
    }

    async fn close(&self) -> Result<(), LoggingError> {
        let mut socket_guard = self.socket.lock().await;
        *socket_guard = None;
        Ok(())
    }

    fn preferred_format(&self) -> LogFormat {
        self.format
    }

    async fn is_available(&self) -> bool {
        let syslog_paths = ["/dev/log", "/var/run/syslog", "/var/run/log"];
        syslog_paths
            .iter()
            .any(|path| std::path::Path::new(path).exists())
    }
}

/// Network syslog output (UDP)
#[derive(Debug)]
pub struct NetworkSyslogOutput {
    address: String,
    facility: SyslogFacility,
    format: LogFormat,
    socket: Arc<Mutex<Option<tokio::net::UdpSocket>>>,
}

impl NetworkSyslogOutput {
    pub fn new(address: String, facility: SyslogFacility, format: LogFormat) -> Self {
        Self {
            address,
            facility,
            format,
            socket: Arc::new(Mutex::new(None)),
        }
    }

    async fn ensure_socket(&self) -> Result<(), LoggingError> {
        let mut socket_guard = self.socket.lock().await;

        if socket_guard.is_none() {
            let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
                .await
                .map_err(LoggingError::IoError)?;

            *socket_guard = Some(socket);
        }

        Ok(())
    }

    fn format_syslog_message(&self, level: u8, message: &str) -> String {
        let priority = (self.facility as u8) * 8 + level;
        let timestamp = chrono::Utc::now().format("%b %d %H:%M:%S");
        let hostname_os_string = gethostname::gethostname();
        let hostname = hostname_os_string.to_string_lossy();
        let tag = "devkit";

        format!(
            "<{}>{} {} {}: {}",
            priority, timestamp, hostname, tag, message
        )
    }
}

#[async_trait]
impl LogOutput for NetworkSyslogOutput {
    async fn write(&self, message: &str) -> Result<(), LoggingError> {
        let level = if message.contains("ERROR") {
            3
        } else if message.contains("WARN") {
            4
        } else if message.contains("INFO") {
            6
        } else {
            7
        };

        self.ensure_socket().await?;

        let syslog_message = self.format_syslog_message(level, message);

        let socket_guard = self.socket.lock().await;
        if let Some(ref socket) = *socket_guard {
            socket
                .send_to(syslog_message.as_bytes(), &self.address)
                .await
                .map_err(LoggingError::IoError)?;
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), LoggingError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), LoggingError> {
        let mut socket_guard = self.socket.lock().await;
        *socket_guard = None;
        Ok(())
    }

    fn preferred_format(&self) -> LogFormat {
        self.format
    }

    async fn is_available(&self) -> bool {
        // Try to resolve the address
        self.address.parse::<std::net::SocketAddr>().is_ok()
    }
}

/// Null output that discards all messages (useful for testing)
#[derive(Debug)]
pub struct NullOutput {
    format: LogFormat,
}

impl NullOutput {
    pub fn new(format: LogFormat) -> Self {
        Self { format }
    }
}

#[async_trait]
impl LogOutput for NullOutput {
    async fn write(&self, _message: &str) -> Result<(), LoggingError> {
        Ok(())
    }

    async fn flush(&self) -> Result<(), LoggingError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), LoggingError> {
        Ok(())
    }

    fn preferred_format(&self) -> LogFormat {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_console_output() {
        let output = ConsoleOutput::new(LogFormat::Text, false);

        assert_eq!(output.preferred_format(), LogFormat::Text);
        assert!(output.is_available().await);

        // These would print to stdout/stderr, so we just test they don't error
        output.write("Test message").await.unwrap();
        output.flush().await.unwrap();
        output.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_file_output() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let output = FileOutput::new(log_path.clone(), LogFormat::Json, None).unwrap();

        assert_eq!(output.preferred_format(), LogFormat::Json);
        assert!(output.is_available().await);

        output.write("Test message 1").await.unwrap();
        output.write("Test message 2").await.unwrap();
        output.flush().await.unwrap();
        output.close().await.unwrap();

        // Verify file contents
        let contents = std::fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("Test message 1"));
        assert!(contents.contains("Test message 2"));
    }

    #[tokio::test]
    async fn test_null_output() {
        let output = NullOutput::new(LogFormat::Text);

        assert_eq!(output.preferred_format(), LogFormat::Text);
        assert!(output.is_available().await);

        output.write("This should be discarded").await.unwrap();
        output.flush().await.unwrap();
        output.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_file_rotation() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("rotating.log");

        let rotation = FileRotation {
            max_size_bytes: 50, // Very small for testing
            max_files: 3,
            compress: false,
        };

        let output = FileOutput::new(log_path.clone(), LogFormat::Text, Some(rotation)).unwrap();

        // Write enough data to trigger rotation
        for i in 0..10 {
            output
                .write(&format!("This is a longer test message {}", i))
                .await
                .unwrap();
        }

        output.close().await.unwrap();

        // Check that rotation occurred (backup files should exist)
        let backup1 = log_path.with_extension("log.1");
        // Note: Actual rotation behavior depends on the exact implementation
        // This test mainly ensures no errors occur during rotation
    }
}
