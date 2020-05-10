> Proxy handler for asynchronous streams.

This library allows for writing reverse proxies in rust. It's built on top of [async-std](https://github.com/async-rs/async-std) and uses [async-uninet](https://github.com/xpepermint/async-uninet) which enables unified handling of asynchronous TCP & Unix streams.

## Example

Program:

```rs
let address = SocketAddr::from_str("127.0.0.1:4444").await.unwrap(); // or `unix:`
let listener = Listener::bind(&address).await.unwrap();

while let Some(stream) = listener.incoming().next().await {
    let stream = stream.unwrap();

    task::spawn(async move {
        let mut router = Router::new(stream);
        let mut router = Router::new(stream);
        router.parse_request().await.unwrap();
        router.write_request_header("Host", "jsonplaceholder.typicode.com:80"); // override header
        router.relay_request().await.unwrap();
        router.parse_response().await.unwrap();
        router.write_response_header("Status", "fast"); // override header
        router.relay_response().await.unwrap();
    })
}
```

Execute:

```sh
$ curl -N \
  -H "Host: typicode" \
  -H "Content-Type: application/json" \
  http://localhost:4444/posts
```
