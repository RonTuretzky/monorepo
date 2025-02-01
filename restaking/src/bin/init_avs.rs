use commonware_restaking::eigenlayer::init_avs_registry_service;
use eigen_logging::{init_logger, log_level::LogLevel};
use eigen_services_avsregistry::{chaincaller::AvsRegistryServiceChainCaller, AvsRegistryService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with INFO level
    init_logger(LogLevel::Info);
    
    println!("Initializing AVS registry service...");
    
    // Initialize the AVS registry service
    init_avs_registry_service().await;
    
    // The service is now initialized and ready to use
    println!("AVS Registry Service initialized successfully!");

    // let quorum_numbers: Vec<u8> = vec![1];
    // let operators_state = avs_registry_service.get_operators_avs_state_at_block(20227142, &quorum_numbers).await?;
    // println!("operators_state: {:?}", operators_state);
    
    Ok(())
} 