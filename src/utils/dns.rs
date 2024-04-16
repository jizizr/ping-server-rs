use hickory_resolver::{
    config::*,
    proto::rr::{RData, RecordType},
    TokioAsyncResolver,
};
use std::net::IpAddr;

use super::BoxError;

async fn ip_resolve(host: &str, record_type: RecordType) -> Result<IpAddr, BoxError> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let response = resolver.lookup(host, record_type).await?;
    Ok(
        match response
            .records()
            .iter()
            .find(|r| r.record_type() == record_type)
            .ok_or("No record found".to_string())?
            .data()
            .ok_or("No data found")?
        {
            RData::A(ipv4) => IpAddr::V4(ipv4.0),
            RData::AAAA(ipv6) => IpAddr::V6(ipv6.0),
            _ => return Err("No A or AAAA record found".into()),
        },
    )
}

pub async fn parse2ip(ip_str: &str, record_type: RecordType) -> Result<IpAddr, BoxError> {
    match ip_str.parse::<IpAddr>() {
        Ok(addr) => Ok(addr),
        Err(_) => ip_resolve(ip_str, record_type).await,
    }
}
