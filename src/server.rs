use std::future::Future;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc,
};

pub async fn process_client<F, Fut>(client_stream: TcpStream, handler: F)
where
    F: Fn(String) -> Fut + Send + 'static,
    Fut: Future<Output = String> + Send + 'static,
{
    let (client_reader, client_writer) = client_stream.into_split();
    let (msg_tx, msg_rx) = mpsc::channel::<String>(100);

    // 从客户端读取的异步子任务
    let mut read_task = tokio::spawn(async move {
        read_from_client(client_reader, msg_tx, handler).await;
    });

    // 向客户端写入的异步子任务
    let mut write_task = tokio::spawn(async move {
        write_to_client(client_writer, msg_rx).await;
    });

    if tokio::try_join!(&mut read_task, &mut write_task).is_err() {
        eprintln!("read_task/write_task terminated");
        read_task.abort();
        write_task.abort();
    };
}

/// 从客户端读取
async fn read_from_client<F, Fut>(reader: OwnedReadHalf, msg_tx: mpsc::Sender<String>, handler: F)
where
    F: Fn(String) -> Fut + Send + 'static,
    Fut: Future<Output = String>,
{
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut buf = String::new();
    loop {
        match buf_reader.read_line(&mut buf).await {
            Err(_e) => {
                eprintln!("read from client error");
                break;
            }
            // 遇到了EOF
            Ok(0) => {
                println!("client closed");
                break;
            }
            Ok(n) => {
                buf.pop();
                let mut content = buf.drain(..).as_str().to_string();
                println!("read {} bytes from client. content: {}", n, content);
                content = handler(content).await;
                if msg_tx.send(content).await.is_err() {
                    eprintln!("receiver closed");
                    break;
                }
            }
        }
    }
    println!("read_from_client terminated");
}

/// 写给客户端
async fn write_to_client(writer: OwnedWriteHalf, mut msg_rx: mpsc::Receiver<String>) {
    let mut buf_writer = tokio::io::BufWriter::new(writer);
    while let Some(mut str) = msg_rx.recv().await {
        str.push('\n');
        if let Err(e) = buf_writer.write_all(str.as_bytes()).await {
            eprintln!("write to client failed: {}", e);
            break;
        }
        // 确保数据被发送到客户端
        if let Err(e) = buf_writer.flush().await {
            eprintln!("failed to flush write buffer: {}", e);
            break;
        }
        println!("write to client: {}", str);
    }
}
