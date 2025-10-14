//! Channel system demonstration
//!
//! This example shows how to use the Bulu channel system for
//! communication between goroutines.

use bulu::interpreter::{Interpreter, Value};
use bulu::types::primitive::TypeId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulu Channel System Demo");
    println!("========================");

    let mut interpreter = Interpreter::new();

    // Create an unbuffered channel
    println!("\n1. Creating unbuffered channel...");
    let unbuffered_channel = interpreter.make_channel(TypeId::Int32, None)?;
    println!("Created unbuffered channel: {:?}", unbuffered_channel);

    // Create a buffered channel with capacity 5
    println!("\n2. Creating buffered channel with capacity 5...");
    let buffered_channel = interpreter.make_channel(TypeId::String, Some(5))?;
    println!("Created buffered channel: {:?}", buffered_channel);

    // Test channel operations
    println!("\n3. Testing channel operations...");

    // Get channel registry for direct operations
    let channel_registry = interpreter.get_channel_registry();

    if let Value::Channel(channel_id) = buffered_channel {
        // Test sending values
        println!("Sending values to buffered channel...");

        let mut reg = channel_registry.lock().unwrap();
        if let Some(channel) = reg.get_mut(channel_id) {
            // Send some values
            for i in 1..=3 {
                let value = bulu::types::primitive::RuntimeValue::Int32(i);
                match channel.try_send(value)? {
                    bulu::runtime::channels::SendResult::Ok => {
                        println!("  Sent value: {}", i);
                    }
                    bulu::runtime::channels::SendResult::Closed => {
                        println!("  Channel closed");
                        break;
                    }
                    bulu::runtime::channels::SendResult::WouldBlock => {
                        println!("  Channel would block");
                        break;
                    }
                }
            }

            println!("Channel length: {}", channel.len());
            println!("Channel capacity: {}", channel.capacity());

            // Receive values
            println!("Receiving values from buffered channel...");
            loop {
                match channel.try_receive()? {
                    bulu::runtime::channels::ChannelResult::Ok(value) => {
                        println!("  Received: {:?}", value);
                    }
                    bulu::runtime::channels::ChannelResult::Closed => {
                        println!("  Channel closed");
                        break;
                    }
                    bulu::runtime::channels::ChannelResult::WouldBlock => {
                        println!("  No more values");
                        break;
                    }
                }
            }

            // Close the channel
            println!("Closing channel...");
            channel.close()?;
            println!("Channel closed successfully");
        }
    }

    // Test channel directions
    println!("\n4. Testing channel directions...");
    let channel = bulu::runtime::channels::Channel::new_buffered(TypeId::Int32, 2);

    let send_only = channel.send_only();
    let receive_only = channel.receive_only();

    println!("Original channel direction: {:?}", channel.direction());
    println!("Send-only channel direction: {:?}", send_only.direction());
    println!(
        "Receive-only channel direction: {:?}",
        receive_only.direction()
    );

    // Test send-only channel
    let value = bulu::types::primitive::RuntimeValue::Int32(42);
    match send_only.try_send(value)? {
        bulu::runtime::channels::SendResult::Ok => {
            println!("Successfully sent to send-only channel");
        }
        _ => println!("Failed to send to send-only channel"),
    }

    // Test receive-only channel
    match receive_only.try_receive()? {
        bulu::runtime::channels::ChannelResult::Ok(value) => {
            println!("Received from receive-only channel: {:?}", value);
        }
        _ => println!("No value received from receive-only channel"),
    }

    println!("\n5. Channel system demo completed successfully!");

    Ok(())
}
