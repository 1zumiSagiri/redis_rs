use bytes::Bytes;
use dashmap::DashMap;
use mini_redis::{Connection, Frame};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Thread-safe reference Arc
type SharedDb = Arc<DashMap<String, Bytes>>;

#[tokio::main]
#[instrument]
async fn main() {
    // Setup logger
    tracing_subscriber::registry()
        .with(fmt::Layer::default())
        .init();

    let tcp_addr = "127.0.0.1:6666";
    let listener = TcpListener::bind(tcp_addr).await.unwrap();
    info!("Listening on {}", tcp_addr);

    // shard_amount must be a power of 2 o/w causes panic
    let size_slices:usize = 4;
    assert!(size_slices.is_power_of_two());
    let shared_db = Arc::new(DashMap::with_shard_amount(size_slices));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        info!("Accepted connection from {:?}", socket.peer_addr().unwrap());

        // Clone handle
        let shared_db = shared_db.clone();

        // Process each socket concurrently
        // 'static spawn lifetime & async move
        tokio::spawn(async move {
            process(socket, shared_db).await;
        });
    }
}

#[instrument]
async fn process(socket: TcpStream, shared_db: SharedDb) {
    use mini_redis::Command::{self, Get, Set};

    info!("Func process started");
    let mut connection = Connection::new(socket);

    info!("Read frame from established connection");
    while let Some(frame) = connection.read_frame().await.unwrap() {
        debug!("Received frame: {:?}", frame);

        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // value: Vec<u8>
                let (k, v) = (cmd.key().to_string(), cmd.value().to_vec().into());
                debug!("Inserted key-value pair ({:?}, {:?})  into db", k, v);
                shared_db.insert(k, v);
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                match shared_db.get(cmd.key()) {
                    Some(value) => {
                        debug!("Query key {:?} from db, got value {:?}", cmd.key(), value);
                        Frame::Bulk(value.clone().into())
                    }
                    None => {
                        debug!("Query key {:?} from db, got None", cmd.key());
                        Frame::Null
                    }
                }
            }
            unknown_cmd => panic!("unimplemented {:?}", unknown_cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}

