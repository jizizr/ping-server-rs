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
        tokio::spawn(async move {
            server::process_client(client_stream, handler).await;
        });
    }
}
