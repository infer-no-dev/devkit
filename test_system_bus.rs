//! Simple test for system bus functionality

use devkit::system_bus::{SystemBus, SystemComponent, SystemEvent};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing System Bus Directly");
    println!("==============================");

    // Create system bus
    let system_bus = Arc::new(SystemBus::new());

    // Register a component
    let (handle, mut receiver) = system_bus
        .register_component(SystemComponent::ShellManager)
        .await;

    // Start a background task to keep receiver alive
    let _receiver_task = tokio::spawn(async move {
        println!("📨 Receiver task started");
        while let Some(message) = receiver.recv().await {
            println!("📨 Received message: {:?}", message.event);
        }
        println!("📨 Receiver task ended");
    });

    // Small delay to ensure receiver task is running
    sleep(Duration::from_millis(10)).await;

    // Test direct bus publish
    println!("🚀 Testing direct bus publish...");
    let test_event = SystemEvent::ShellCommandExecuted {
        command: "echo test".to_string(),
        exit_code: 0,
        output: "test".to_string(),
    };

    match handle.publish(test_event).await {
        Ok(_) => println!("✅ Direct publish succeeded"),
        Err(e) => println!("❌ Direct publish failed: {}", e),
    }

    // Test multiple publishes
    println!("🔄 Testing multiple publishes...");
    for i in 1..=3 {
        let event = SystemEvent::ShellCommandExecuted {
            command: format!("echo test{}", i),
            exit_code: 0,
            output: format!("test{}", i),
        };

        match handle.publish(event).await {
            Ok(_) => println!("✅ Publish {} succeeded", i),
            Err(e) => println!("❌ Publish {} failed: {}", i, e),
        }

        sleep(Duration::from_millis(100)).await;
    }

    println!("🏁 System bus test completed");
    Ok(())
}
