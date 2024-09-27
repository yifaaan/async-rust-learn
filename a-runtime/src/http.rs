use crate::future::{Future, PollState};
use crate::{runtime, Runtime};
use mio::{Interest, Token};
use std::io::{ErrorKind, Read, Write};
pub struct Http;

impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    /// read data from stream and put it in this
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
        }
    }

    fn write_request(&mut self) {
        // 连接web服务器
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        // 发送get请求
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("First poll - start operation");
            self.write_request();

            // 向OS注册读事件
            runtime::registry()
                .register(self.stream.as_mut().unwrap(), Token(0), Interest::READABLE)
                .unwrap();
            // return PollState::NotReady;
        }

        let mut buff = vec![0u8; 4096];

        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                // 服务器的响应信息全部发送完毕
                Ok(0) => {
                    // 存储响应信息
                    let s = String::from_utf8_lossy(&self.buffer);
                    // task: leaf-future 执行完成
                    break PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    self.buffer.extend(&buff[0..n]);
                    // 尝试继续读响应信息
                    continue;
                }
                // 服务器没准备好数据，需要继续等待
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break PollState::NotReady;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => {
                    panic!("{e:?}");
                }
            }
        }
    }
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        \r\n"
    )
}
