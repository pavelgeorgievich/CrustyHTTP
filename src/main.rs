mod server;
use crate::server::Server;


#[tokio::main]
async fn main(){
    let port = 8081;
    let ip = "127.0.0.1";
    let server = Server::new(ip,port);

    server.listen().await.expect("TODO: panic message");
}