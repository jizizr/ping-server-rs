use serde::Deserialize;

use crate::utils::{dns, ping, BoxError};

#[derive(Deserialize)]
struct Msg {
    method: String,
    host: String,
    port: Option<u16>,
    record_type: String,
}

async fn msg_handler(raw_msg: String) -> Result<String, BoxError> {
    let msg = serde_json::from_str::<Msg>(&raw_msg)?;
    let ip = dns::parse2ip(&msg.host, msg.record_type.parse()?).await?;
    let timeout_duration = std::time::Duration::from_secs(1);
    match msg.method.as_str() {
        "ping" => {
            let duration = ping::ping(ip, 2, timeout_duration).await?;
            Ok(format!("{} ms", duration.as_secs_f32() * 1000.0))
        }
        "tcping" => {
            let port = msg.port.unwrap_or(80);
            let duration = ping::tcping(ip, port, timeout_duration).await?;
            Ok(format!("{} ms", duration.as_secs_f32() * 1000.0))
        }
        _ => Ok("".to_string()),
    }
}

pub async fn handler(raw_msg: String) -> String {
    match msg_handler(raw_msg).await {
        Ok(msg) => msg,
        Err(e) => e.to_string(),
    }
}
