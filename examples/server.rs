use core::panic;
use std::{collections::HashMap, sync::Arc, sync::Mutex};

use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
 
use tokio::net::{TcpListener, TcpStream};

type DB = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let tcp = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Arc::new(Mutex::new(HashMap::new()));
    
    loop {
        let (socket, _) = tcp.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(socket, db).await;
        }); 
    }
} 

async fn process(stream: TcpStream, db: DB) {
    let mut conn = Connection::new(stream);
    while let Some(frame) = conn.read_frame().await.unwrap() {
        let res = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("no {:?}", cmd),
        };
        conn.write_frame(&res).await.unwrap();
    }
}
