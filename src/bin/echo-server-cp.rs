use tokio::io;
use tokio::net::TcpStream;

#[tokio::main]
async fn main () -> io::Result<()> {
    // let (mut reader, mut writer) = io::split(socket);
    loop {
        let mut socket = TcpStream::connect("127.0.0.1:6667").await?;

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();

            if io::copy(&mut reader, &mut writer).await.is_err() {
                eprintln!("failed to read data from socket");
            }
        });
    }
}