use commonware_restaking::eigenlayer::init_avs_registry_service;
use eigen_logging::{init_logger, log_level::LogLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with INFO level
    init_logger(LogLevel::Info);
    
    println!("Initializing AVS registry service...");
    
    // Initialize the AVS registry service
    let avs_registry_service = init_avs_registry_service().await?;
    
    // The service is now initialized and ready to use
    println!("AVS Registry Service initialized successfully!");
    
    Ok(())
} 