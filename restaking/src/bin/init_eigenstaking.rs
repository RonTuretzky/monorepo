use commonware_restaking::eigenlayer::EigenStakingClient;
use eigen_logging::{init_logger, log_level::LogLevel};
use alloy_primitives::address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with INFO level
    init_logger(LogLevel::Info);
    
    println!("Initializing EigenStaking client...");
    println!("\nConfiguration:");
    println!("HTTP Endpoint: https://withered-convincing-meadow.quiknode.pro/89fd706450ed0a8279f87c01e52ae78d9b308ce7");
    println!("WS Endpoint: wss://withered-convincing-meadow.quiknode.pro/89fd706450ed0a8279f87c01e52ae78d9b308ce7");
    println!("Registry Coordinator Address: 0xeCd099fA5048c3738a5544347D8cBc8076E76494");
    println!("Operator State Retriever Address: D5D7fB4647cE79740E6e83819EFDf43fa74F8C31");
    println!("Registry Coordinator Deploy Block: 20227142");
    
    // Initialize the EigenStaking client with default values
    let client = EigenStakingClient::new(
        String::from("https://withered-convincing-meadow.quiknode.pro/89fd706450ed0a8279f87c01e52ae78d9b308ce7"),
        String::from("wss://withered-convincing-meadow.quiknode.pro/89fd706450ed0a8279f87c01e52ae78d9b308ce7"),
        address!("0xeCd099fA5048c3738a5544347D8cBc8076E76494").into(),
        20227142,
    ).await?;
    
    println!("\nRetrieving operator states...");
    client.get_operator_states().await?;
    
    println!("\nEigenStaking client initialized and operator states retrieved successfully!");
    Ok(())
} 