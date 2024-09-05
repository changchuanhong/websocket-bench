use tokio::net::TcpListener;
use tokio::io::BufStream;
use ws_tool::{
    codec::{default_handshake_handler, AsyncBytesCodec, FrameConfig},
    ServerBuilder,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let _jh = tokio::spawn(async move {
            let buf_size = 1024 * 16;
            let (mut read, mut write) = ServerBuilder::async_accept(
                stream,
                default_handshake_handler,
                |_req, stream| {
                    let stream = BufStream::with_capacity(buf_size, buf_size, stream);
                    let config = FrameConfig {
                        mask_send_frame: false,
                        resize_size: buf_size,
                        ..Default::default()
                    };
                    Ok(AsyncBytesCodec::new_with(stream, config))
                },
            )
            .await
            .unwrap()
            .split();
            loop {
                let msg = read.receive().await.unwrap();
                if msg.code.is_data() {
                    write.send(msg).await.unwrap();
                    write.flush().await.unwrap();
                } else if msg.code.is_close() {
                    break;
                } else {
                    println!("Unrecognized code {:?}, aborting...", msg.code);
                    break;
                }
            }
        });
    }
}
