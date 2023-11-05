mod response;
mod router;
mod thread_pool;

use {
    crate::router::{Method, Router},
    crate::thread_pool::ThreadPool,
    std::{
        net::TcpListener,
        sync::{Arc, Mutex},
    },
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let thread_pool = ThreadPool::new(4);

    let mut router = Router::new();

    router.insert(Method::GET, "/", |res| {
        res.sendfile(200, "static/index.html").unwrap();
    });

    router.insert(Method::GET, "/hello", |res| {
        res.sendfile(200, "static/index.html").unwrap();
    });

    let router = Arc::new(Mutex::new(router));

    for stream in listener.incoming() {
        let router = router.clone();
        thread_pool.execute(move || {
            router
                .lock()
                .unwrap()
                .route_client(stream.unwrap())
                .unwrap();
        });
    }
}
