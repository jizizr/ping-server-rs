use crate::utils::{dns, ping, BoxError};
use ping_server_rs::model::{Answer, Target};

async fn msg_handler(raw_msg: String) -> Result<Answer, BoxError> {
    let msg = serde_json::from_str::<Target>(&raw_msg)?;
    let timeout_duration = std::time::Duration::from_secs(1);
    let timeout_duration_http = std::time::Duration::from_secs(3);
    match msg.method.as_str() {
        "ping" => {
            let ip = dns::parse2ip(&msg.host, msg.record_type.parse()?).await?;
            let answer = ping::ping(ip, timeout_duration, 6).await?;
            Ok(answer)
        }
        "tcping" => {
            let ip = dns::parse2ip(&msg.host, msg.record_type.parse()?).await?;
            let port = msg.port.unwrap_or(80);
            let duration = ping::tcping((ip, port), timeout_duration, 6).await?;
            Ok(duration)
        }
        "http" => {
            let answer = ping::httping(&msg.host, timeout_duration_http, 6).await?;
            Ok(answer)
        }
        _ => Ok(Answer::new()),
    }
}

pub async fn handler(raw_msg: String) -> String {
    match msg_handler(raw_msg).await {
        Ok(msg) => serde_json::to_string(&msg).unwrap(),
        Err(e) => {
            let mut msg = Answer::new();
            msg.error = Some(e.to_string());
            serde_json::to_string(&msg).unwrap()
        }
    }
}
