// 初始化日志：统一 [时间戳] [级别] 信息 格式，输出到控制台
pub fn init_logger() {
    use std::io::Write;
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(buf, "[{}] [{}] {}", ts, record.level(), record.args())
        })
        .init();
}
