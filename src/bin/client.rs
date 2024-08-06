use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

/// Commands that can be sent to the single channel from multi requesters
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

/// Mgr use to send results back to requesters
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // cmd manager with buffer size 32
    let (tx, mut rx) = mpsc::channel(32);

    let mgr = tokio::spawn(async move {
        // Connect to server to send cmds
        let mut client = client::connect("127.0.0.1:6666").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            if cfg!(debug_assertions) {
                eprintln!("debug: Got cmd {:?}", cmd);
            }
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "hello".to_string(),
            resp: resp_tx
        };

        tx.send(cmd).await.unwrap();
        let res = resp_rx.await;
        println!("Got {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx
        };

        tx2.send(cmd).await.unwrap();
        let res = resp_rx.await;
        println!("Got {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    mgr.await.unwrap();
}
