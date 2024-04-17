use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::utils::{dns, ping, BoxError};

#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub method: String,
    pub host: String,
    pub port: Option<u16>,
    pub record_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Answer {
    pub error: Option<String>,
    pub success: u16,
    pub fail: u16,
    pub avg_time: f32,
    total_time: f32,
    pub loss: f32,
}

impl Answer {
    pub fn new() -> Self {
        Self {
            error: None,
            success: 0,
            fail: 0,
            avg_time: 0.0,
            total_time: 0.0,
            loss: 100.0,
        }
    }

    pub fn add_success(&mut self, duration: Duration) {
        self.success += 1;
        self.total_time += duration.as_secs_f32() * 1000.0;
    }

    pub fn add_fail(&mut self) {
        self.fail += 1;
    }

    pub fn over(mut self) -> Self {
        if self.success != 0 {
            self.avg_time = self.total_time as f32 / self.success as f32;
            let total = self.success + self.fail;
            self.loss = self.fail as f32 / total as f32 * 100.0;
        } else {
            self.error = Some("TimeOut".to_string());
        }
        self
    }
}

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
