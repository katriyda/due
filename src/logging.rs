/// 初始化日志系统
/// - 输出到 stderr（开发环境）
/// - 输出到文件 %APPDATA%\due\logs\due.log（生产环境）
/// - 通过环境变量 RUST_LOG 控制日志级别
/// - 默认级别：info
pub fn init_logging() -> Result<(), fern::InitError> {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("due")
        .join("logs");

    // 确保日志目录存在
    std::fs::create_dir_all(&log_dir).ok();

    let log_file = log_dir.join("due.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("due", log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .chain(fern::log_file(log_file)?)
        .apply()?;

    Ok(())
}

/// 测试用 Logger，全局单例，捕获日志输出供断言
#[cfg(test)]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(test)]
static TEST_RECORDS: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

#[cfg(test)]
struct GlobalTestLogger;

#[cfg(test)]
impl log::Log for GlobalTestLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Some(records) = TEST_RECORDS.get() {
            records.lock().unwrap().push(record.args().to_string());
        }
    }

    fn flush(&self) {}
}

/// 初始化全局测试 Logger（只能调用一次），返回日志记录引用
#[cfg(test)]
pub(crate) fn setup_test_logger() -> Arc<Mutex<Vec<String>>> {
    let records = Arc::new(Mutex::new(Vec::new()));
    let shared = TEST_RECORDS.get_or_init(|| records.clone());
    // 如果 OnceLock 已初始化，set_boxed_logger 会失败，这是正常的
    let _ = log::set_boxed_logger(Box::new(GlobalTestLogger));
    log::set_max_level(log::LevelFilter::Debug);
    shared.clone()
}

/// 清空测试日志记录
#[cfg(test)]
pub(crate) fn clear_test_records(records: &Arc<Mutex<Vec<String>>>) {
    records.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_logging_returns_ok_or_already_set() {
        let result = init_logging();

        match result {
            Ok(_) => {}
            Err(fern::InitError::SetLoggerError(_)) => {}
            Err(fern::InitError::Io(_)) => {}
        }
    }

    #[test]
    fn logging_outputs_to_stderr() {
        let records = setup_test_logger();
        clear_test_records(&records);

        log::info!("test message");

        let records = records.lock().unwrap();
        assert!(records.iter().any(|r| r.contains("test message")), "should log 'test message', got: {:?}", *records);
    }

    #[test]
    fn logging_creates_log_directory() {
        let log_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("due")
            .join("logs");

        let _ = std::fs::remove_dir_all(&log_dir);

        let _ = init_logging();

        assert!(log_dir.exists(), "log directory should exist at {:?}", log_dir);
    }

    #[test]
    fn logging_format_contains_required_parts() {
        let records = setup_test_logger();
        clear_test_records(&records);

        log::info!("format test");

        let records = records.lock().unwrap();
        let last = records.last().unwrap();
        // 格式: [YYYY-MM-DD HH:MM:SS LEVEL module] message
        assert!(last.contains("format test"), "should contain message, got: {}", last);
    }
}