mod handler;
mod server;
mod utils;
use crate::handler::handler;
use tokio::net::TcpListener;
#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:8888").await.unwrap();
    while let Ok((client_stream, client_addr)) = server.accept().await {
        println!("accept client: {}", client_addr);
        // 每接入一个客户端的连接请求，都分配一个子任务，
        // 如果客户端的并发数量不大，为每个客户端都分配一个thread，
        // 然后在thread中创建tokio runtime，处理起来会更方便
        tokio::spawn(async move {
            server::process_client(client_stream, handler).await;
        });
    }
}
