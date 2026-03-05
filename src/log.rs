/*
 * Copyright (c) 2025 Beijing Volcano Engine Technology Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use chrono::Utc;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};

const FILE_DATE_FORMAT: &str = "%Y-%m-%d";
const FILE_SUFFIX: &str = ".log";

#[derive(Debug, Clone)]
pub(crate) struct DualRollingConfig {
    pub(crate) directory: PathBuf,
    pub(crate) file_name_prefix: String,
    pub(crate) max_file_size: usize,
    pub(crate) rotate_daily: bool,
    pub(crate) max_file_number: usize,
}

impl Default for DualRollingConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("./logs"),
            file_name_prefix: "app".to_string(),
            max_file_size: 30 * 1024 * 1024,
            rotate_daily: true,
            max_file_number: 0,
        }
    }
}

impl DualRollingConfig {
    pub(crate) fn new(directory: impl AsRef<Path>, file_name_prefix: impl Into<String>) -> Self {
        let mut config = Self::default();
        config.directory = directory.as_ref().to_path_buf();
        config.file_name_prefix = file_name_prefix.into();
        config
    }

    pub(crate) fn max_file_size(mut self, max_file_size: usize) -> Self {
        self.max_file_size = max_file_size;
        self
    }

    pub(crate) fn rotate_daily(mut self, rotate_daily: bool) -> Self {
        self.rotate_daily = rotate_daily;
        self
    }

    pub(crate) fn max_file_number(mut self, max_file_number: usize) -> Self {
        self.max_file_number = max_file_number;
        self
    }

    pub(crate) fn build(self) -> std::io::Result<DualRollingWriter> {
        DualRollingWriter::new(self)
    }
}


pub(crate) struct DualRollingWriter {
    pub(crate) config: DualRollingConfig,
    pub(crate) current_file: Option<File>,
    pub(crate) current_size: usize,
    pub(crate) current_index: u32,
    pub(crate) current_date: String,
}

impl DualRollingWriter {
    pub(crate) fn new(config: DualRollingConfig) -> std::io::Result<Self> {
        std::fs::create_dir_all(&config.directory)?;

        let now = Utc::now().format(FILE_DATE_FORMAT).to_string();
        let latest_index = Self::find_latest_index(&config, &now);
        let mut writer = Self {
            config,
            current_file: None,
            current_size: 0,
            current_index: latest_index,
            current_date: now,
        };
        writer.open_new_file()?;
        writer.cleanup_old_files();
        Ok(writer)
    }

    fn find_latest_index(config: &DualRollingConfig, date: &String) -> u32 {
        let full_prefix = format!("{}.{}.", config.file_name_prefix, date);
        let mut latest_index = 0u32;
        if let Ok(items) = std::fs::read_dir(&config.directory) {
            for item in items.flatten() {
                let file_name = item.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&full_prefix) && file_name.ends_with(FILE_SUFFIX) {
                    if let Some(index) = file_name.strip_prefix(&full_prefix).and_then(|s| s.strip_suffix(FILE_SUFFIX)) {
                        if let Ok(index) = index.parse::<u32>() {
                            if index > latest_index {
                                latest_index = index;
                            }
                        }
                    }
                }
            }
        }
        latest_index
    }

    fn make_file_path(&self) -> PathBuf {
        let filename = format!(
            "{}.{}.{}{}",
            self.config.file_name_prefix,
            self.current_date,
            self.current_index,
            FILE_SUFFIX,
        );
        self.config.directory.join(filename)
    }

    fn open_new_file(&mut self) -> std::io::Result<()> {
        let path = self.make_file_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let size = file.metadata()?.len();
        self.current_file = Some(file);
        self.current_size = size as usize;
        Ok(())
    }

    fn cleanup_old_files(&self) {
        if self.config.max_file_number == 0 {
            return;
        }

        let prefix = format!("{}.", self.config.file_name_prefix);
        let mut log_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.config.directory) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&prefix) && name.ends_with(FILE_SUFFIX) {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            log_files.push((entry.path(), modified));
                        }
                    }
                }
            }
        }

        if log_files.len() > self.config.max_file_number {
            log_files.sort_by_key(|(_, t)| *t);
            let to_remove = log_files.len() - self.config.max_file_number;
            for (path, _) in log_files.iter().take(to_remove) {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    fn rotate(&mut self, incoming_bytes: usize) -> std::io::Result<()> {
        let now = Utc::now().format(FILE_DATE_FORMAT).to_string();;
        let mut need_rotate = false;
        if self.config.rotate_daily && self.current_date != now {
            self.current_date = now;
            self.current_index = 0;
            need_rotate = true;
        }

        if self.current_size + incoming_bytes > self.config.max_file_size {
            if !need_rotate {
                self.current_index += 1;
            }
            need_rotate = true;
        }

        if need_rotate {
            if let Some(mut file) = self.current_file.take() {
                let _ = file.flush();
            }
            self.open_new_file()?;
            self.cleanup_old_files();
        }
        Ok(())
    }
}

impl Write for DualRollingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.rotate(buf.len())?;
        match self.current_file.as_mut() {
            None => Err(Error::new(ErrorKind::Other, "log file not open")),
            Some(file) => {
                let written = file.write(buf)?;
                self.current_size += written;
                Ok(written)
            }
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.rotate(buf.len())?;
        match self.current_file.as_mut() {
            None => Err(Error::new(ErrorKind::Other, "log file not open")),
            Some(file) => {
                file.write_all(buf)?;
                self.current_size += buf.len();
                Ok(())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.current_file.as_mut() {
            None => Ok(()),
            Some(file) => file.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::log::DualRollingConfig;
    use tracing::log::{error, info, warn};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Layer};

    #[test]
    fn test_dual_rolling_writer() {
        let rolling_writer = DualRollingConfig::new("temp/logs", "app.log")
            .max_file_size(5 * 1024 * 1024)    // 单文件最大 50MB
            .rotate_daily(true)        // 按天滚动
            .max_file_number(30)      // 最多保留 30 个文件
            .build().unwrap();
        let (non_blocking, _guard) = tracing_appender::non_blocking(rolling_writer);

        tracing_subscriber::fmt()
            .with_line_number(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_writer(non_blocking)
            .with_env_filter(EnvFilter::new("info"))
            .with_ansi(false).init();

        info!("hello world");
        warn!("hi world");
    }

    #[test]
    fn test_multi_logs() {
        let common_writer = DualRollingConfig::new("temp/logs", "app")
            .max_file_size(5 * 1024 * 1024)    // 单文件最大 50MB
            .rotate_daily(true)        // 按天滚动
            .max_file_number(30)      // 最多保留 30 个文件
            .build().unwrap();

        let (common_writer, _guard1) = tracing_appender::non_blocking(common_writer);

        let access_writer = DualRollingConfig::new("temp/logs", "access")
            .max_file_size(5 * 1024 * 1024)    // 单文件最大 50MB
            .rotate_daily(true)        // 按天滚动
            .max_file_number(30)      // 最多保留 30 个文件
            .build().unwrap();

        let (access_writer, _guard2) = tracing_appender::non_blocking(access_writer);

        let common_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_writer(common_writer)
            .with_filter(EnvFilter::new("common=info"));

        let access_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_writer(access_writer)
            .with_filter(EnvFilter::new("access=warn"));

        tracing_subscriber::registry()
            .with(common_layer)
            .with(access_layer)
            .init();

        info!(target: "common", "{} {}", "hello", "world" );
        info!(target: "access","hi world");
        warn!(target: "common", "hello world");
        warn!(target: "access", "hi world");
        error!(target: "common", "{} {}", "hello", "world");
        error!(target: "access", "hi world");
    }
}
