use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {

    let mut client = client::connect("127.0.0.1:6666").await?;

    client.set("hello", "world".into()).await?;
    client.set("Rust", "good".into()).await?;

    let result1 = client.get("hello").await?;
    let result2 = client.get("Rust").await?;

    println!("Result from server={:?}", result1);
    println!("Result from server={:?}", result2);


    Ok(())
}