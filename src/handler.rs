use crate::utils::{dns, ping, BoxError};
use ping_server_rs::model::{Answer, Target};

async fn msg_handler(raw_msg: String) -> Result<Answer, BoxError> {
    let msg = serde_json::from_str::<Target>(&raw_msg)?;
    let ip = dns::parse2ip(&msg.host, msg.record_type.parse()?).await?;
    let timeout_duration = std::time::Duration::from_secs(1);

    match msg.method.as_str() {
        "ping" => {
            let answer = ping::ping(ip, timeout_duration, 6).await?;
            Ok(answer)
        }
        "tcping" => {
            let port = msg.port.unwrap_or(80);
            let duration = ping::tcping((ip, port), timeout_duration, 6).await?;
            Ok(duration)
        }
        _ => Ok(Answer::new()),
    }
}

pub async fn handler(raw_msg: String) -> String {
    match msg_handler(raw_msg).await {
        Ok(msg) => serde_json::to_string(&msg).unwrap(),
        Err(e) => e.to_string(),
    }
}
