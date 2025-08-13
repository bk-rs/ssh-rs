#![cfg(all(feature = "_integration_tests", feature = "tokio"))]

use std::sync::Arc;
use async_ssh2_lite::{AsyncSession, TokioTcpStream};
use tokio::io::AsyncReadExt;
use std::time::Duration;

async fn get_test_session() -> Result<AsyncSession<TokioTcpStream>, Box<dyn std::error::Error + Send + Sync>> {
    let host = std::env::var("SSH_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("SSH_SERVER_PORT")
        .unwrap_or_else(|_| "22".to_string())
        .parse()?;
    
    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse()?;
    let mut session = AsyncSession::<TokioTcpStream>::connect(addr, None).await?;
    session.handshake().await?;
    
    if let Ok(password) = std::env::var("SSH_PASSWORD") {
        let username = std::env::var("SSH_USERNAME").unwrap_or_else(|_| "test".to_string());
        session.userauth_password(&username, &password).await?;
    } else if let Ok(key_path) = std::env::var("SSH_PRIVATEKEY_PATH") {
        let username = std::env::var("SSH_USERNAME").unwrap_or_else(|_| "test".to_string());
        session.userauth_pubkey_file(&username, None, std::path::Path::new(&key_path), None).await?;
    } else {
        return Err("No authentication method available".into());
    }
    
    Ok(session)
}

#[tokio::test]
async fn test_single_channel_baseline() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Baseline test - single channel should always work
    let session = get_test_session().await?;
    
    let mut channel = session.channel_session().await?;
    channel.exec("echo baseline").await?;
    
    let mut output = String::new();
    channel.read_to_string(&mut output).await?;
    channel.close().await?;
    
    println!("Baseline output: {}", output.trim());
    assert!(output.contains("baseline"));
    
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn test_two_channels_sequential() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Two channels used one after another - should work
    let session = Arc::new(get_test_session().await?);
    
    // First channel
    {
        let mut channel1 = session.channel_session().await?;
        channel1.exec("echo first").await?;
        
        let mut output1 = String::new();
        channel1.read_to_string(&mut output1).await?;
        channel1.close().await?;
        
        println!("First channel: {}", output1.trim());
        assert!(output1.contains("first"));
    }
    
    // Second channel
    {
        let mut channel2 = session.channel_session().await?;
        channel2.exec("echo second").await?;
        
        let mut output2 = String::new();
        channel2.read_to_string(&mut output2).await?;
        channel2.close().await?;
        
        println!("Second channel: {}", output2.trim());
        assert!(output2.contains("second"));
    }
    
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn test_two_channels_interleaved() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Two channels with interleaved operations - this tests our fix
    let session = Arc::new(get_test_session().await?);
    
    let mut channel1 = session.channel_session().await?;
    let mut channel2 = session.channel_session().await?;
    
    // Start both commands
    channel1.exec("echo first && sleep 0.1 && echo done1").await?;
    channel2.exec("echo second && sleep 0.1 && echo done2").await?;
    
    // Read from both channels
    let mut output1 = String::new();
    let mut output2 = String::new();
    
    // This should work with our fix - the async runtime will serialize properly
    let result1 = channel1.read_to_string(&mut output1);
    let result2 = channel2.read_to_string(&mut output2);
    
    // Wait for both to complete
    let (r1, r2) = tokio::try_join!(result1, result2)?;
    
    channel1.close().await?;
    channel2.close().await?;
    
    println!("Channel 1 output: {}", output1.trim());
    println!("Channel 2 output: {}", output2.trim());
    
    assert!(output1.contains("first") && output1.contains("done1"));
    assert!(output2.contains("second") && output2.contains("done2"));
    
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn test_stress_many_channels() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Stress test with many channels
    let session = Arc::new(get_test_session().await?);
    
    let mut channels = Vec::new();
    
    // Create multiple channels
    for i in 0..5 {
        let mut channel = session.channel_session().await?;
        channel.exec(&format!("echo channel{}", i)).await?;
        channels.push(channel);
    }
    
    // Read from all channels
    let mut outputs = Vec::new();
    for (i, channel) in channels.iter_mut().enumerate() {
        let mut output = String::new();
        channel.read_to_string(&mut output).await?;
        println!("Channel {} output: {}", i, output.trim());
        assert!(output.contains(&format!("channel{}", i)));
        outputs.push(output);
    }
    
    // Close all channels
    for channel in channels.iter_mut() {
        channel.close().await?;
    }
    
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn test_data_integrity() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Test that data doesn't get corrupted between channels
    let session = Arc::new(get_test_session().await?);
    
    let mut channel1 = session.channel_session().await?;
    let mut channel2 = session.channel_session().await?;
    
    // Send different data patterns to each channel
    let data1 = "AAAAAAAAAAAAAAAAAAAA";
    let data2 = "BBBBBBBBBBBBBBBBBBBB";
    
    channel1.exec(&format!("echo {}", data1)).await?;
    channel2.exec(&format!("echo {}", data2)).await?;
    
    let mut output1 = String::new();
    let mut output2 = String::new();
    
    // Read both outputs
    let (r1, r2) = tokio::try_join!(
        channel1.read_to_string(&mut output1),
        channel2.read_to_string(&mut output2)
    )?;
    
    channel1.close().await?;
    channel2.close().await?;
    
    println!("Channel 1: {}", output1.trim());
    println!("Channel 2: {}", output2.trim());
    
    // Verify data integrity - no cross-contamination
    assert!(output1.contains(data1));
    assert!(output2.contains(data2));
    assert!(!output1.contains(data2), "Data contamination detected!");
    assert!(!output2.contains(data1), "Data contamination detected!");
    
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Test error conditions with multiple channels
    let session = Arc::new(get_test_session().await?);
    
    let mut channel1 = session.channel_session().await?;
    let mut channel2 = session.channel_session().await?;
    
    // One good command, one bad command
    channel1.exec("echo good").await?;
    channel2.exec("this_command_does_not_exist").await?;
    
    let mut output1 = String::new();
    let mut output2 = String::new();
    
    let result1 = channel1.read_to_string(&mut output1).await;
    let result2 = channel2.read_to_string(&mut output2).await;
    
    // Both should complete (even if one has error output)
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    println!("Good command output: {}", output1.trim());
    println!("Bad command output: {}", output2.trim());
    
    assert!(output1.contains("good"));
    // Bad command might produce stderr or empty output, both are okay
    
    channel1.close().await?;
    channel2.close().await?;
    
    Ok(())
}