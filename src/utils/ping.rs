use super::BoxError;
use rand::random;
use std::{net::IpAddr, ops::Add};
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

pub async fn tcping(host: IpAddr, port: u16, timeout_duration: Duration) -> Result<Duration, BoxError> {
    let start = Instant::now();
    let _ = timeout(timeout_duration, TcpStream::connect((host, port))).await?;
    Ok(start.elapsed())
}

pub async fn ping(host: IpAddr, times: u16, timeout_duration: Duration) -> Result<Duration, BoxError> {
    let mut pinger = CONFIG_V4.pinger(host, PingIdentifier(random())).await;
    pinger.timeout(timeout_duration);
    let mut avg = Duration::new(0, 0);
    for idx in 0..times {
        let (_, duration) = pinger.ping(PingSequence(idx), &PAYLOAD).await?;
        avg = avg.add(duration);
    }
    Ok(avg / times.into())
}
