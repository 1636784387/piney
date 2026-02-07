use futures_util::StreamExt;
use std::io::Write;
use tauri::command;

#[command]
pub async fn download_large_file(url: String, file_path: String) -> Result<(), String> {
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()));
    }

    let mut file =
        std::fs::File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;
    }

    Ok(())
}

#[command]
pub async fn restart_server(
    state: tauri::State<'_, std::sync::Arc<crate::ServerControl>>,
) -> Result<(), String> {
    state.restart_trigger.notify_one();
    Ok(())
}
