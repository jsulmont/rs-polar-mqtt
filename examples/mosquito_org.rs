use polar_mqtt::{Client, QoS};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

fn main() -> polar_mqtt::Result<()> {
    println!("Initializing MQTT monitor...");

    let client_id = format!("RustMonitor_{}", Uuid::new_v4());
    println!("Client ID: {}", client_id);

    let (state_tx, state_rx) = mpsc::channel();
    let state_tx = Arc::new(Mutex::new(state_tx));

    let mut client = Client::new(
        &client_id,
        move |msg| {
            let payload = msg.payload();

            // Attempt UTF-8 conversion, fallback to hex display for binary data
            let preview = match String::from_utf8(payload[..payload.len().min(20)].to_vec()) {
                Ok(s) => format!("{}{}", s, if payload.len() > 20 { "..." } else { "" }),
                Err(_) => format!("{:02X?}", &payload[..payload.len().min(10)]),
            };

            println!(
                "Topic: {:<50} | Length: {:>4} | Preview: {}",
                msg.topic(),
                payload.len(),
                preview
            );
        },
        move |state| {
            let _ = state_tx.lock().unwrap().send(state);
        },
    )?;

    println!("Connecting to test.mosquitto.org...");
    client.connect("test.mosquitto.org", 1883)?;

    match state_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(state) => println!("Connection state: {:?}", state),
        Err(_) => {
            println!("Timeout waiting for connection");
            return Ok(());
        }
    }

    println!("Subscribing to all topics (#)...");
    let sub_handle = client.subscribe("#", QoS::AtMostOnce)?;
    println!("Subscribed successfully");

    println!("\nMonitoring messages (Press Ctrl+C to stop)...");
    println!("────────────────────────────────────────────────────────────────────────────────");

    // Setup Ctrl-C handling
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        // Ignore send errors during shutdown
        let _ = tx.send(());
    })
    .expect("Error setting Ctrl-C handler");

    // Wait for Ctrl-C
    let _ = rx.recv();

    println!("\nReceived Ctrl-C, cleaning up...");
    client.unsubscribe(sub_handle)?;
    println!("Unsubscribed. Exiting.");

    Ok(())
}
