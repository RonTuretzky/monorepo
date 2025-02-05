use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_logging::get_logger;
use eigen_services_operatorsinfo::operator_info::OperatorInfoService;
use eigen_services_operatorsinfo::operatorsinfo_inmemory::OperatorInfoServiceInMemory;
use eigen_utils::middleware::operatorstateretriever::OperatorStateRetriever;
use eigen_common::get_provider;

use alloy_primitives::{Address, address};
use alloy_provider::{Provider, RootProvider};
use alloy_network::Ethereum;

use url::Url;
use std::sync::Arc;

/// source: https://github.com/Layr-Labs/eigenlayer-middleware
/// Contracts from middleware are supposed to be deployed for each AVS but
/// OperatorStateRetriever looks generic for everyone.
const OPERATOR_STATE_RETRIEVER_ADDRESS: Address =
    address!("D5D7fB4647cE79740E6e83819EFDf43fa74F8C31");

pub struct EigenStakingClient {
    http_endpoint: String,
    registry_coordinator_address: Address,
    registry_coordinator_deploy_block: u64,
    operator_info_service: Arc<OperatorInfoServiceInMemory>,
}

impl EigenStakingClient {
    pub async fn new(
        http_endpoint: String,
        ws_endpoint: String,
        registry_coordinator_address: Address,
        registry_coordinator_deploy_block: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {

        let avs_registry_reader = AvsRegistryChainReader::new(
            get_logger().clone(),
            registry_coordinator_address,
            OPERATOR_STATE_RETRIEVER_ADDRESS,
            http_endpoint.clone(),
        ).await?;
        
        let (operator_info_service, _rx) = OperatorInfoServiceInMemory::new(
            get_logger(),
            avs_registry_reader.clone(),
            ws_endpoint.clone(),
        ).await?;
        Ok(Self {
            http_endpoint,
            registry_coordinator_address,
            registry_coordinator_deploy_block,
            operator_info_service: Arc::new(operator_info_service),
        })
    }

    pub async fn get_operator_states(&self) -> Result<(), Box<dyn std::error::Error>> {
        let provider: RootProvider<_, Ethereum> = RootProvider::new_http(Url::parse(&self.http_endpoint)?);
        let current_block_number = provider.get_block_number().await?;
        
        println!("\nClient Configuration:");
        println!("Registry Coordinator Address: {}", self.registry_coordinator_address);
        println!("Operator State Retriever Address: {}", OPERATOR_STATE_RETRIEVER_ADDRESS);
        println!("Registry Coordinator Deploy Block: {}", self.registry_coordinator_deploy_block);
        println!("Current Block Number: {}", current_block_number);
        
        self.operator_info_service
            .query_past_registered_operator_events_and_fill_db(
                self.registry_coordinator_deploy_block,
                current_block_number
            ).await?;
        let provider = get_provider(&self.http_endpoint);
        let operator_state_retriever = OperatorStateRetriever::new(OPERATOR_STATE_RETRIEVER_ADDRESS, provider);
        let quorum_numbers: Vec<u8> = vec![0];
        let operators_state = operator_state_retriever
            .getOperatorState_0(
                self.registry_coordinator_address,
                quorum_numbers.into(),
                current_block_number.try_into().unwrap()
            )
            .call()
            .await?
            ._0;

        println!("\nOperator States Summary:");
        println!("Total number of quorums: {}", operators_state.len());
        println!("\nDetailed Operators Information:");
        
        let mut total_operators = 0;
        let mut total_stake = 0u128;
        
        for (i, operators) in operators_state.iter().enumerate() {
            println!("\nQuorum {} Details:", i);
            println!("Number of operators in quorum: {}", operators.len());
            
            let mut quorum_stake = 0u128;
            for op in operators {
                total_operators += 1;
                quorum_stake += u128::try_from(op.stake).unwrap_or_default();
                
                println!("\n  Operator Address: {}", op.operator);
                println!("  Stake: {} wei", op.stake);
                if let Ok(info) = self.operator_info_service.get_operator_info(op.operator).await {
                    println!("  Operator Info: {:?}", info);
                }
                if let Ok(socket) = self.operator_info_service.get_operator_socket(op.operator).await {
                    println!("  Operator Socket: {:?}", socket);
                }
            }
            total_stake += quorum_stake;
            println!("  Total stake in quorum {}: {} wei", i, quorum_stake);
        }
        
        println!("\nGlobal Statistics:");
        println!("Total number of operators across all quorums: {}", total_operators);
        println!("Total stake across all quorums: {} wei", total_stake);
        
        Ok(())
    }
}
