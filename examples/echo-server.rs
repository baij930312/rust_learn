use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1::6142").await.unwrap();
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = vec![0, 128];

            match socket.read(&mut buf).await {
                Ok(0) => return,
                Ok(n) => {
                    if socket.write_all(&buf[..n]).await.is_err(){
                         return;
                    };
                }
                Err(_) => return,
            }
        });
    }
}