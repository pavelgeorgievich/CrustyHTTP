mod server;
use crate::server::{Route, Server};

struct MockRecord {
    id: &'static str,
    message: &'static str,
}
fn call_to_the_database_mock() -> [MockRecord; 1] {
    [MockRecord {
        id: "1234",
        message: "TestMessage",
    }]
}

#[tokio::main]
async fn main() {
    let port = 8081;
    let ip = "127.0.0.1";
    let server = Server::new(ip, port);
    let route = Route { path: "test" };

    server
        .routes(&[route])
        .listen()
        .await
        .expect("TODO: panic message");
}
