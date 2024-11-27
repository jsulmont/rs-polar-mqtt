use polar_mqtt::{Client, Message, QoS};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> polar_mqtt::Result<()> {
    println!("Initializing API...");

    // Create channel for message communication
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(Mutex::new(tx));

    // Generate test topic
    let test_topic = "test/topic".to_string();
    let test_topic_clone = test_topic.clone();

    // Create client with callbacks
    let mut client = Client::new(
        "TestClient",
        // Message callback - note that message is now passed by reference
        move |msg| {
            if msg.topic() == test_topic_clone {
                // Clone message to send through channel since we only have a reference
                let message = Message::new(msg.topic(), msg.payload().to_vec())
                    .with_qos(msg.qos())
                    .with_retain(msg.is_retained());
                let _ = tx.lock().unwrap().send(message);
            }
        },
        // State callback remains the same
        |state| println!("Connection state changed to: {:?}", state),
    )?;

    println!("Connecting to broker...");
    client.connect("broker.emqx.io", 1883)?;

    // Give connection time to establish
    std::thread::sleep(Duration::from_secs(1));

    println!("Subscribing to topic: {}", test_topic);
    client.subscribe(&test_topic, QoS::AtLeastOnce)?;

    // Wait for subscription to establish
    std::thread::sleep(Duration::from_secs(1));

    // Prepare test message
    let test_payload = "{\"test\":true}";
    println!("Publishing test message...");
    println!("Topic: {}", test_topic);
    println!("Payload: {}", test_payload);

    let msg = Message::new(&test_topic, test_payload)
        .with_qos(QoS::AtLeastOnce)
        .with_retain(false);

    let msg_id = client.publish(&msg)?;
    println!("Published message id: {}", msg_id);

    // Wait for message with timeout
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(msg) => {
            let received_payload = String::from_utf8_lossy(msg.payload());
            if msg.topic() == test_topic && received_payload == test_payload {
                println!("Successfully received matching message!");
            } else {
                println!("Received non-matching message:");
                println!("Expected topic: {}, got: {}", test_topic, msg.topic());
                println!(
                    "Expected payload: {}, got: {}",
                    test_payload, received_payload
                );
            }
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            println!("Timeout waiting for message");
        }
        Err(e) => {
            println!("Error receiving message: {}", e);
        }
    }

    Ok(())
}
