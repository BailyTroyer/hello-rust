pub mod messenger {
    tonic::include_proto!("messenger");
}

use futures::stream::Stream;
use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use messenger::messenger_client::MessengerClient;
use messenger::MessageRequest;

fn chat_requests_iter() -> impl Stream<Item = MessageRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| MessageRequest {
        msg: format!("msg {:02}", i),
    })
}

async fn bidirectional_streaming_chat(client: &mut MessengerClient<Channel>, num: usize) {
    let in_stream = chat_requests_iter().take(num);

    let response = client.chat(in_stream).await.unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        println!("\treceived msg: `{}`", received.msg);
    }
}

async fn chat_throttle(client: &mut MessengerClient<Channel>, dur: Duration) {
    let in_stream = chat_requests_iter().throttle(dur);

    let response = client.chat(in_stream).await.unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        println!("\treceived msg: `{}`", received.msg);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessengerClient::connect("http://[::1]:50051")
        .await
        .unwrap();

    println!("Streaming chat:");
    tokio::time::sleep(Duration::from_secs(1)).await; //do not mess server println functions

    // chat stream that sends 17 requests then graceful end that connection
    println!("\r\nBidirectional stream chat:");
    bidirectional_streaming_chat(&mut client, 17).await;

    // chat stream that sends up to `usize::MAX` requets. One request each 2s.
    // Exiting client with CTRL+C demonstrate how to distinguish broken pipe from
    //graceful client disconnection (above example) on the server side.
    println!("\r\nBidirectional stream chat (kill client with CTLR+C):");
    chat_throttle(&mut client, Duration::from_secs(2)).await;

    Ok(())
}
