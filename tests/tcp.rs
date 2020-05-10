// use async_std::prelude::*;
// use async_std::task;
// use rawproxy::{Listener, SocketAddr, Stream, Router};

// #[async_std::test]
// async fn starts_inet_server() {
//     let address = SocketAddr::from_str("127.0.0.1:4445").await.unwrap();
//     let listener = Listener::bind(&address).await.unwrap();

//     task::spawn(async move {
//         Stream::connect(&address).await.unwrap();
//     });

//     while let Some(stream) = listener.incoming().next().await {
//         let stream = stream?;
//         let options = options.clone();

//         task::spawn(async move {
//             let mut router = Router::from_stream(stream);
//             router.set_options(options);
//             router.resolve().await.unwrap();
//         });
//     }

//     assert!(result);
// }
