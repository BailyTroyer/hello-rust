use tonic::{transport::Server, Request, Response, Status, Streaming};

use futures::Stream;
use std::{error::Error, io::ErrorKind, pin::Pin};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use messenger::messenger_server::{Messenger, MessengerServer};
use messenger::{MessageRequest, MessageResponse};

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<MessageResponse, Status>> + Send>>;

pub mod messenger {
    tonic::include_proto!("messenger");
}

fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}

#[derive(Debug, Default)]
pub struct MyMessenger {}

#[tonic::async_trait]
impl Messenger for MyMessenger {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<MessageRequest>>,
    ) -> EchoResult<Self::ChatStream> {
        println!("Messenger::chat");

        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => tx
                        .send(Ok(MessageResponse { msg: v.msg }))
                        .await
                        .expect("working rx"),
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // here you can handle special case when client
                                // disconnected in unexpected way
                                eprintln!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }

                        match tx.send(Err(err)).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was droped
                        }
                    }
                }
            }
            println!("\tgoodbye");
        });

        // echo just write the same data that was received
        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(out_stream) as Self::ChatStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let messenger = MyMessenger::default();

    Server::builder()
        .add_service(MessengerServer::new(messenger))
        .serve(addr)
        .await?;

    Ok(())
}
