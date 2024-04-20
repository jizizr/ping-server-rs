mod handler;
mod server;
mod utils;
use crate::handler::handler;
use tokio::net::TcpListener;
#[tokio::main]
async fn main() {
    // 从命令行参数中获取监听地址
    let addr = std::env::args()
        .nth(1)
        .unwrap_or("0.0.0.0:55555".to_string());
    println!("server listen on: {}", addr);
    let server = TcpListener::bind(addr).await.unwrap();
    while let Ok((client_stream, client_addr)) = server.accept().await {
        println!("accept client: {}", client_addr);
        tokio::spawn(async move {
            server::process_client(client_stream, handler).await;
        });
    }
}
