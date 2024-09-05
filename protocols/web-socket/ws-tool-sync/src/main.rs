use ws_tool::{
    codec::{default_handshake_handler, BytesCodec},
    stream::BufStream,
    ServerBuilder,
};


fn main() {
    let listener = std::net::TcpListener::bind("0.0.0.0:9000").unwrap();
    let pool = threadpool::ThreadPool::new(8);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream.set_nodelay(true).unwrap();
        pool.execute(move || {
            let buf_size = 1024 * 16;
            let (mut r, mut w) =
                        ServerBuilder::accept(stream, default_handshake_handler, |req, stream| {
                            let stream = BufStream::with_capacity(buf_size, buf_size, stream);
                            BytesCodec::factory(req, stream)
                        })
                        .unwrap()
                        .split();
            loop {
                match r.receive() {
                        Ok(msg) => {
                            if msg.code.is_data() {
                                w.send(msg).unwrap();
                            } else if msg.code.is_close() {
                                break;
                            } else {
                                println!("Unrecognized code {:?}, aborting...", msg.code);
                                break;
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
            }
        });
    }
}
