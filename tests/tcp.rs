use async_std::prelude::*;
use async_std::task;
use rawproxy::{Listener, SocketAddr, Router, Stream};

#[async_std::test]
async fn starts_inet_server() {

    let address = SocketAddr::from_str("127.0.0.1:4444").await.unwrap();
    let listener = Listener::bind(&address).await.unwrap();

    task::spawn(async move {
        let mut req = Stream::connect(&address).await.unwrap();
        req.write(b"GET /posts HTML HTTP/1.1\r\n").await.unwrap();
        req.write(b"Host: fake\r\n").await.unwrap();
        req.write(b"\r\n").await.unwrap();
        req.flush().await.unwrap();
        let mut data = String::new();
        req.read_to_string(&mut data).await.unwrap();
    });

    while let Some(stream) = listener.incoming().next().await {
        let stream = stream.unwrap();

        let mut router = Router::new(stream);
        router.parse_request().await.unwrap();
        router.set_request_header("Host", "jsonplaceholder.typicode.com:80"); // override
        router.relay_request().await.unwrap();
        router.parse_response().await.unwrap();
        router.relay_response().await.unwrap();

        break;
    }
}
