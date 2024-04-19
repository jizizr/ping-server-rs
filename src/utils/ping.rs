use super::BoxError;
use futures::stream::{FuturesUnordered, StreamExt};
use ping_server_rs::model::Answer;
use rand::random;
use std::net::IpAddr;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::{
    net::TcpStream,
    time::{timeout, Duration, Instant},
};

lazy_static::lazy_static! {
    static ref CONFIG_V4: Client = Client::new(&Config::builder().kind(ICMP::V4).build()).unwrap();
    static ref CONFIG_V6: Client = Client::new(&Config::builder().kind(ICMP::V6).build()).unwrap();
}

const PAYLOAD: [u8; 56] = [0; 56];

async fn tcping_once(
    host: (IpAddr, u16),
    timeout_duration: Duration,
) -> Result<Duration, BoxError> {
    let start = Instant::now();
    let _ = timeout(timeout_duration, TcpStream::connect(host)).await?;
    Ok(start.elapsed())
}

pub async fn tcping(
    host: (IpAddr, u16),
    timeout_duration: Duration,
    times: u16,
) -> Result<Answer, BoxError> {
    let mut futures = FuturesUnordered::new();
    let mut answer = Answer::new();

    for _ in 0..times {
        futures.push(tokio::spawn(tcping_once(host, timeout_duration)));
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok(duration) => match duration {
                Ok(duration) => answer.add_success(duration),
                Err(_) => answer.add_fail(),
            },
            Err(e) => return Err(e.into()),
        }
    }

    Ok(answer.over())
}

pub async fn ping(ip: IpAddr, timeout_duration: Duration, times: u16) -> Result<Answer, BoxError> {
    let mut futures = FuturesUnordered::new();
    let mut answer = Answer::new();

    for idx in 0..times {
        futures.push(tokio::spawn(async move {
            let mut pinger = CONFIG_V4.pinger(ip, PingIdentifier(random())).await;
            pinger.timeout(timeout_duration);
            match pinger.ping(PingSequence(idx), &PAYLOAD).await {
                Ok(duration) => Some(duration),
                Err(_) => None,
            }
        }));
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok(Some((_, duration))) => answer.add_success(duration),
            Ok(None) => answer.add_fail(),
            Err(e) => return Err(e.into()), // 处理可能的错误
        }
    }

    Ok(answer.over()) // 最后处理Answer对象
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[tokio::test]
    async fn test_ping() {
        let host = IpAddr::V4("1.1.1.1".parse().unwrap());
        let times = 6;
        let timeout_duration = Duration::from_secs(2);
        let answer = ping(host, timeout_duration, times).await.unwrap();
        println!("{:?}", serde_json::to_string(&answer).unwrap());
    }

    #[tokio::test]
    async fn test_tcping() {
        let host = (IpAddr::V4("1.1.1.1".parse().unwrap()), 80);
        let times = 6;
        let timeout_duration = Duration::from_secs(2);
        let answer = tcping(host, timeout_duration, times).await.unwrap();
        println!("{:?}", serde_json::to_string(&answer).unwrap());
    }
}
