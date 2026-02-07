use axum::{response::IntoResponse, Json};
use std::{thread, time::Duration};

pub async fn restart() -> impl IntoResponse {
    // 启动后台线程，延迟退出进程
    // Docker 容器通常配置为 restart: always，所以退出进程等同于重启
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        std::process::exit(0);
    });

    Json("Restarting...")
}
