use crate::router::{Route, RouteType, Router};
use crate::threadpool::handle_message;
use std::process;
use std::sync::Arc;
// pyO3 module
use pyo3::prelude::*;
use pyo3::types::PyAny;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[pyclass]
pub struct Server {
    port: usize,
    number_of_threads: usize,
    router: Arc<Router>, //
}

// unsafe impl Send for Server {}

#[pymethods]
impl Server {
    #[new]
    pub fn new() -> Self {
        Self {
            port: 5000,
            number_of_threads: 1,
            router: Arc::new(Router::new()),
        }
    }

    pub fn start(&mut self, py: Python) {
        let url = format!("127.0.0.1:{}", &self.port);
        let router = self.router.clone();
        pyo3_asyncio::tokio::init_multi_thread_once();

        let py_loop = pyo3_asyncio::tokio::run_until_complete(py, async move {
            let listener = TcpListener::bind(url).await.unwrap();
            while let Ok((mut stream, _addr)) = listener.accept().await {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).await.unwrap();

                let route = Route::new(RouteType::Buffer(Box::new(buffer)));
                match router.get_route(route) {
                    Some(a) => tokio::spawn(async move {
                        handle_message(a, stream).await;
                    }),
                    None => {
                        println!("No match found");
                        continue;
                    }
                };
            }

            Ok(())
        });
        match py_loop {
            Ok(_) => {}
            Err(_) => {
                process::exit(1);
            }
        };
    }

    pub fn add_route(&self, route_type: &str, route: String, handler: Py<PyAny>) {
        println!("{} {} ", route_type, route);
        let route = Route::new(RouteType::Route((route, route_type.to_string())));
        self.router.add_route(route_type, route, handler);
    }
}
