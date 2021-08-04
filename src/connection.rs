use std::net::TcpStream;
use tungstenite::WebSocket;

#[derive(Debug)]
pub struct Connection {
    inner: WebSocket<TcpStream>,
}

impl Connection {
    pub fn new() -> Self {
        let (inner, _) = tungstenite::connect("ws://localhost:12250/").unwrap();

        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(disabled)]
    #[test]
    fn websocket_test() {
        let (mut ws, _) = tungstenite::connect("ws://localhost:12250/").unwrap();

        loop {
            let msg = ws.read_message().unwrap();
            println!("{}", msg);
        }
    }
}
