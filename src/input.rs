use std::io::*;

use messenger::messenger_client::MessengerClient;
use messenger::MessageRequest;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use tonic::transport::Channel;

pub mod messenger {
    tonic::include_proto!("messenger");
}

fn start_reading_stdin_lines(sender: Sender<String>, runtime: Handle) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line_buf = String::new();
        while let Ok(_) = stdin.read_line(&mut line_buf) {
            let line = line_buf.trim_end().to_string();
            line_buf.clear();
            let sender2 = sender.clone();

            runtime.spawn(async move {
                let result = sender2.send(line).await;
                if let Err(error) = result {
                    println!("start_reading_stdin_lines send error: {:?}", error);
                }
            });
        }
    });
}

fn start_reading_grpc_response(sender: Sender<String>, client: &mut MessengerClient<Channel>) {
    std::thread::spawn(move || {
        // let in_stream = chat_requests_iter().take(num);

        let response = client.chat(in_stream).await.unwrap();

        let mut resp_stream = response.into_inner();

        while let Some(received) = resp_stream.next().await {
            let received = received.unwrap();
            println!("\treceived msg: `{}`", received.msg);
        }
    });
}

// fn start_activity_until_shutdown(watch_sender: tokio::sync::watch::Sender<bool>) {
//     tokio::spawn(async move {
//         tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
//         println!("exiting after a signal...");
//         let result = watch_sender.send(true);
//         if let Err(error) = result {
//             println!("watch_sender send error: {:?}", error);
//         }
//     });
// }

async fn read_input(
    mut line_receiver: tokio::sync::mpsc::Receiver<String>,
    mut watch_receiver: tokio::sync::watch::Receiver<bool>,
) {
    let mut client = MessengerClient::connect("http://[::1]:50051")
        .await
        .unwrap();

    loop {
        tokio::select! {
            Some(line) = line_receiver.recv() => {
                // process the input
                match line.as_str() {
                    "/q" => {
                        println!("quitting");
                        break;
                    },
                    message => {
                        // println!("{:?}", message);
                        client.chat(tokio_stream::once(MessageRequest {
                            msg: String::from(message),
                        })).await.unwrap();
                    }
                }
            }
            Ok(_) = watch_receiver.changed() => {
                println!("shutdown");
                break;
            }
        }
    }

    // let response = client.chat(stream).await.unwrap();

    // let mut resp_stream = response.into_inner();

    // while let Some(received) = resp_stream.next().await {
    //     let received = received.unwrap();
    //     println!("{}", received.msg);
    // }
}

#[tokio::main]
async fn main() {
    let (line_sender, line_receiver) = tokio::sync::mpsc::channel(1);
    start_reading_stdin_lines(line_sender, tokio::runtime::Handle::current());

    let (watch_sender, watch_receiver) = tokio::sync::watch::channel(false);
    // this will send a shutdown signal at some point
    // start_activity_until_shutdown(watch_sender);

    read_input(line_receiver, watch_receiver).await;
}
