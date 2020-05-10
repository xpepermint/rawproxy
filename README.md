> Proxy handler for asynchronous streams.

This library allows for writing reverse proxies in rust. It's built with [async-uninet](https://github.com/xpepermint/async-uninet) which enables unified handling of asynchronous TCP & Unix streams.

## Example

Program:

```rs
let options = Arc::new({
    let mut options = RouterOptions::default();
    options.set_target("typicode", "jsonplaceholder.typicode.com:80"); // override hosts
    options
});
let address = SocketAddr::from_str("127.0.0.1:4444").await.unwrap(); // tcp or unix socket
let listener = Listener::bind(&address).await.unwrap();

while let Some(stream) = listener.incoming().next().await {
    let stream = stream.unwrap();
    let options = options.clone();

    task::spawn(async move {
        let mut router = Router::from_stream(stream); // proxy requests
        router.set_options(options);
        router.resolve().await.unwrap();
    });
}
```

Execute:

```sh
$ curl -N \
  -H "Host: typicode" \
  -H "Content-Type: application/json" \
  http://localhost:4444/posts
```
