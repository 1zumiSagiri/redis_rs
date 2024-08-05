use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
#[instrument]
async fn main(){
    // Setup logger
    tracing_subscriber::registry()
        .with(fmt::Layer::default())
        .init();

    let listener = TcpListener::bind("127.0.0.1:6666").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // Process each socket concurrently
        // 'static spawn lifetime & async move
        tokio::spawn(async move {
            process(socket).await;
        });
    }

}

#[instrument]
async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    info!("Func process started");
    let mut db = HashMap::new();
    let mut connection = Connection::new(socket);

    info!("Read frame from established connection");
    while let Some(frame) = connection.read_frame().await.unwrap() {
        debug!("Received frame: {:?}", frame);

        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // value: Vec<u8>
                let (k, v) = (cmd.key().to_string(), cmd.value().to_vec());
                debug!("Inserted key-value pair ({:?}, {:?})  into db", k, v);
                db.insert(k, v);
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => match db.get(cmd.key()) {
                Some(value) => {
                    debug!("Query key {:?} from db, got value {:?}", cmd.key(), value);
                    Frame::Bulk(value.clone().into())
                }
                None => {
                    debug!("Query key {:?} from db, got None", cmd.key());
                    Frame::Null
                }
            },
            unknown_cmd => panic!("unimplemented {:?}", unknown_cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
