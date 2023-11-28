#[tokio::main]
async fn main() {
    let task = tokio::spawn(async {
        println!("This is an asynchronous task.");
    });
    // no need to await it
    // task.await.unwrap();
    // loop {} it starts immediately

    //--------------------------------------------
    use tokio::time::{sleep, timeout, Duration};
    sleep(Duration::from_secs(2)).await;
    println!("done waiting");

    //--------------------------------------------
    let _ = timeout(Duration::from_secs(2), async {
        println!("timeout is over");
    })
    .await;

    //--------------------------------------------
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::select;
    use anyhow::{Result, anyhow, Context};
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        // tohle nefici
        // let (mut socket, _) = select! {
        //     conn = listener.accept() => conn.context("failed to accept connection"),
        //     _ = timeout(Duration::from_secs(5), async {}) => Err(anyhow!("timeout")),
        // }.unwrap();
        
        let (mut socket, _) = timeout(tokio::time::Duration::from_secs(5), async { listener.accept().await} ).await.expect("a").expect("b");

        listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // In a real-world application, you should handle errors appropriately
            match socket.read(&mut buf).await {
                Ok(_) => {
                    // Echo back to the client
                    socket.write_all(&buf).await.unwrap();
                }
                Err(e) => println!("Failed to read from socket: {:?}", e),
            }
        });
    }

    //--------------------------------------------

    // multiple connections from presentation...

    //--------------------------------------------
    use tokio::fs::File;

    //use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
    let mut file = File::create("foo.txt").await.unwrap();

    file.write_all(b"Hello, world!").await.unwrap();

    let mut file = File::open("foo.txt").await.unwrap();

    let mut contents = vec![];

    file.read_to_end(&mut contents).await.unwrap();

    println!("File contents: {:?}", String::from_utf8(contents).unwrap());

    //--------------------------------------------
    // use hyper::{Body, Client, Request};
    // //use hyper::rt::Future;
    // use hyper::http::Result;
    // let client = Client::new();
    // let req = Request::builder()
    //     .uri("http://httpbin.org/ip")
    //     .body(Body::empty())?;
    // let res = client.request(req).await.unwrap();

    // println!("Response: {}", res.status());

    //--------------------------------------------

    // tokio mpsc channel.. flume is more flexible; tokio has boardcast
    // tokio has async mutex, possible to hold lock across await points
    // tokio mutex - safer, but slower

    //--------------------------------------------
    //use tokio::select;
    //use tokio::time::{sleep, Duration};
    let future1 = sleep(Duration::from_secs(5));
    let future2 = sleep(Duration::from_secs(10));
    select! {
    _ = future1 => println!("Future 1 completed first"),
    _ = future2 => println!("Future 2 completed first"),
    }

    //--------------------------------------------
    use tokio::sync::mpsc;
    let (tx, mut rx) = mpsc::channel(32);
    let timeout_duration = Duration::from_secs(1);
    // Simulate an external event sending a message
    tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        tx.send("Message from external event").await.unwrap();
    });

    select! {
        Some(message) = rx.recv() => {
            // Handle the message received from the channel
            println!("Received message: {}", message);
        }
        _ = sleep(timeout_duration) => {
            // Handle timeout
            println!(
                "No message received within {} seconds; operation timed out.",
                timeout_duration.as_secs()
            );
        }
    }
}
