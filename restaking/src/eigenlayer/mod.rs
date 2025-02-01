use eigen_services_avsregistry::AvsRegistryService;
use eigen_services_blsaggregation::bls_agg::BlsAggregatorService;
use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_logging::{get_logger, init_logger};
use eigen_logging::logger::Logger;
use eigen_logging::noop_logger::NoopLogger;
use eigen_services_avsregistry::chaincaller::AvsRegistryServiceChainCaller;
use eigen_services_operatorsinfo::operator_info::OperatorInfoService;
use eigen_services_operatorsinfo::operatorsinfo_inmemory::OperatorInfoServiceInMemory;
use eigen_crypto_bls::{Signature, BlsG2Point};
use eigen_types::{
    avs::TaskResponseDigest,
    operator::QuorumThresholdPercentages,
};
use eigen_utils::middleware::operatorstateretriever::OperatorStateRetriever;
use eigen_common::get_provider;

use alloy_primitives::{Address, Bytes, FixedBytes, address};
use alloy_provider::{Provider, RootProvider};
use alloy_network::{Ethereum, Network};
use alloy_transport::Transport;

use url::Url;
use std::time::Duration;
use std::str::FromStr;
use tokio_util::sync::CancellationToken;
use tokio::{task, time::sleep};
use std::collections::HashSet;

/// source: https://github.com/Layr-Labs/eigenlayer-middleware
/// Contracts from middleware are supposed to be deployed for each AVS but
/// OperatorStateRetriever looks generic for everyone.
const OPERATOR_STATE_RETRIEVER_ADDRESS: Address =
    address!("D5D7fB4647cE79740E6e83819EFDf43fa74F8C31");

pub struct EigenStakingClient<T: Transport + std::clone::Clone, N: Network> {
    provider: RootProvider<T, N>,
    registry_coordinator_address: Address,
}

// impl<T: Transport + std::clone::Clone, N: Network> EigenStakingClient<T, N> {
//     pub fn new(
//         provider: RootProvider<T, N>,
//         registry_coordinator_address: Address,
//         operator_state_retriever_address: Option<Address>,
//     ) -> Self {
//         let operator_state_retriever_address = match operator_state_retriever_address {
//             Some(address) => address,
//             None => OPERATOR_STATE_RETRIEVER_ADDRESS,
//         };
//         Self {
//             provider,
//             registry_coordinator_address,
//             operator_state_retriever_address,
//         }
//     }

//     pub async fn get_avs_operators(
//         &self,
//         block_number: u32,
//     ) -> Result<OperatorState, alloy::contract::Error> {
//         let registry_coordinator =
//             RegistryCoordinator::new(self.registry_coordinator_address, self.provider.clone());
//         let operation_state_retriever = OperationStateRetriever::new(
//             self.operator_state_retriever_address,
//             self.provider.clone(),
//         );

//         let builder = registry_coordinator.quorumCount();
//         let quorum_count = builder.call().await?._0;
//         let quorum_numbers: Vec<u8> = Vec::from_iter(1..=quorum_count);
//         let operators_state = operation_state_retriever
//             .getOperatorState_0(
//                 self.registry_coordinator_address,
//                 quorum_numbers.into(),
//                 block_number,
//             )
//             .call()
//             .await?
//             ._0;
//         Ok(OperatorState::new(block_number, operators_state))
//     }

    pub async fn init_avs_registry_service() {
        let registry_coordinator_address: Address = address!("0xeCd099fA5048c3738a5544347D8cBc8076E76494").into();
        let operator_state_retriever_address: Address = address!("D5D7fB4647cE79740E6e83819EFDf43fa74F8C31").into();
        let http_endpoint = String::from("https://eth.llamarpc.com");
        let ws_endpoint = String::from("wss://mainnet.gateway.tenderly.co");
        let provider: RootProvider<_, Ethereum> = RootProvider::new_http(Url::parse(&http_endpoint).unwrap());
        let avs_registry_reader = AvsRegistryChainReader::new(
            get_logger().clone(),
            registry_coordinator_address,
            operator_state_retriever_address,
            http_endpoint.clone(),
        ).await.expect("Failed to create avs registry reader");
        println!("avs_registry_reader: {:?}", avs_registry_reader);
        let (operators_info, _rx) = OperatorInfoServiceInMemory::new(
            get_logger(),
            avs_registry_reader.clone(),
            ws_endpoint,
        ).await.expect("Failed to create operator info service");
        println!("operators_info: {:?}", operators_info);
        operators_info.query_past_registered_operator_events_and_fill_db(20227142, 21738679).await.expect("Failed to query past events");
        let provider = get_provider("https://withered-convincing-meadow.quiknode.pro/89fd706450ed0a8279f87c01e52ae78d9b308ce7");
        let contract = OperatorStateRetriever::new(operator_state_retriever_address, provider);
        let quorum_numbers: Vec<u8> = vec![0];
        let operators_state = match contract.getOperatorState_0(registry_coordinator_address, quorum_numbers.into(), 21738679).call().await {
            Ok(result) => result._0,
            Err(e) => {
                println!("Error getting operator state: {}", e);
                return;
            }
        };
        println!("Number of operator states: {}", operators_state.len());
        println!("\nOperators Details:");
        for (i, operators) in operators_state.iter().enumerate() {
            println!("\nQuorum {}:", i);
            for op in operators {
                println!("\n  Operator Address: {}", op.operator);
                println!("  Stake: {}", op.stake);
                if let Ok(info) = operators_info.get_operator_info(op.operator).await {
                    println!("  Operator Info: {:?}", info);
                }
                if let Ok(info) = operators_info.get_operator_socket(op.operator).await {
                    println!("  Operator Socket: {:?}", info);
                }
            }
        }
        // let operators_state = avs_registry_service.get_operators_avs_state_at_block(20227142, &quorum_numbers).await?;
        // // println!("operators_state: {:?}", operators_state);
        // Ok(avs_registry_service)

    }

    
// }

// pub struct OperatorState {
//     block_number: u32,
//     quorums_operators: Vec<Vec<OperatorStateRetriever::Operator>>,
// }

// #[derive(PartialEq, Eq, Hash)]
// pub struct Operator {
//     address: Address,
//     id: FixedBytes<32>,
// }

// impl OperatorState {
//     fn new(
//         block_number: u32,
//         quorums_operators: Vec<Vec<OperatorStateRetriever::Operator>>,
//     ) -> Self {
//         Self {
//             block_number,
//             quorums_operators,
//         }
//     }

//     pub fn get_block_number(&self) -> u32 {
//         self.block_number
//     }

//     pub fn get_quorum_count(&self) -> usize {
//         self.quorums_operators.len()
//     }

//     pub fn get_operator_set(&self) -> HashSet<Operator> {
//         let mut set = HashSet::new();
//         for quorum_operator_list in &self.quorums_operators {
//             for operator in quorum_operator_list {
//                 set.insert(Operator {
//                     address: operator.operator,
//                     id: operator.operatorId,
//                 });
//             }
//         }
//         set
//     }

//     /// Returns the (OperatorStake,TotalStake) of the provided quorum number.
//     pub fn get_operator_weight(
//         &self,
//         operator_id: FixedBytes<32>,
//         quorum_number: usize,
//     ) -> Option<(U256, U256)> {
//         let quorum_operators = self.quorums_operators.get(quorum_number - 1)?;
//         let mut operator_staked: Option<U256> = None;
//         let mut total_staked: U256 = Uint::from(0);
//         for operator in quorum_operators {
//             let stake = operator.stake;
//             total_staked = total_staked.saturating_add(U256::from(stake));
//             if operator_id == operator.operatorId {
//                 operator_staked = Some(U256::from(stake));
//             }
//         }
//         Some((operator_staked?, total_staked))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use alloy::sol_types::private;
//     use alloy::{providers::ProviderBuilder, sol};
//     use alloy_node_bindings::Anvil;

//     use super::*;
//     use rand::Rng;
//     use std::sync::Arc;

//     sol!(
//         #[allow(missing_docs)]
//         #[sol(rpc)]
//         MockedRegistryCoordinator,
//         "src/eigenlayer/artifacts/registry_coordinator.sol/RegistryCoordinator.json"
//     );
//     sol!(
//         #[allow(missing_docs)]
//         #[sol(rpc)]
//         MockedOperatorStateRetriever,
//         "src/eigenlayer/artifacts/operator_state_retriever.sol/OperatorStateRetriever.json"
//     );

//     #[tokio::test]
//     async fn test_mocked_registry_coordinator() {
//         let anvil = Anvil::new().block_time(1_u64).spawn();
//         let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());
//         let anvil_provider = Arc::new(anvil_provider);
//         let coordinator = MockedRegistryCoordinator::deploy(anvil_provider)
//             .await
//             .unwrap();
//         coordinator
//             .setQuorumCount(3)
//             .send()
//             .await
//             .unwrap()
//             .watch()
//             .await
//             .unwrap();
//         let count = coordinator.quorumCount().call().await.unwrap()._0;
//         assert_eq!(count, 3);
//     }

//     #[tokio::test]
//     async fn test_mocked_operator_state_retriever() {
//         let anvil = Anvil::new().block_time(1_u64).spawn();

//         let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());
//         let anvil_provider = Arc::new(anvil_provider);

//         let coordinator = MockedRegistryCoordinator::deploy(anvil_provider.clone())
//             .await
//             .unwrap();

//         let retriever = MockedOperatorStateRetriever::deploy(anvil_provider)
//             .await
//             .unwrap();
//         retriever
//             .setRegistryCoordinator(*coordinator.address())
//             .send()
//             .await
//             .unwrap()
//             .watch()
//             .await
//             .unwrap();

//         let registry_coordinator_address =
//             retriever._registryCoordinator().call().await.unwrap()._0;
//         assert_eq!(&registry_coordinator_address, coordinator.address());
//     }

//     #[tokio::test]
//     async fn test_eigen_layer_client() {
//         let anvil = Anvil::new().block_time(1_u64).spawn();

//         let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());

//         let mocked_registry_coordinator = MockedRegistryCoordinator::deploy(anvil_provider.clone())
//             .await
//             .unwrap();

//         let mocked_state_retriever = MockedOperatorStateRetriever::deploy(anvil_provider.clone())
//             .await
//             .unwrap();

//         let _ = mocked_registry_coordinator
//             .setQuorumCount(3)
//             .send()
//             .await
//             .unwrap()
//             .watch()
//             .await
//             .unwrap();
//         let count = mocked_registry_coordinator
//             .quorumCount()
//             .call()
//             .await
//             .unwrap()
//             ._0;
//         assert_eq!(count, 3);

//         let operator_1_quorum_1 = generate_operator();
//         let operator_2_quorum_1 = generate_operator();
//         let operator_1_quorum_3 = update_operator_stake(&operator_1_quorum_1);
//         let operator_2_quorum_3 = update_operator_stake(&operator_2_quorum_1);

//         let operators_quorum_1: private::Vec<OperatorStateRetriever::Operator> =
//             vec![operator_1_quorum_1.clone(), operator_2_quorum_1.clone()];
//         let operators_quorum_2: private::Vec<OperatorStateRetriever::Operator> = vec![];
//         let operators_quorum_3: private::Vec<OperatorStateRetriever::Operator> =
//             vec![operator_1_quorum_3, operator_2_quorum_3];

//         mocked_state_retriever
//             .setOperators(1, operators_quorum_1)
//             .send()
//             .await
//             .unwrap()
//             .watch()
//             .await
//             .unwrap();
//         mocked_state_retriever
//             .setOperators(2, operators_quorum_2)
//             .send()
//             .await
//             .unwrap()
//             .watch()
//             .await
//             .unwrap();
//         mocked_state_retriever
//             .setOperators(3, operators_quorum_3)
//             .send()
//             .await
//             .unwrap()
//             .watch()
//             .await
//             .unwrap();

//         let quorum_numbers: Vec<u8> = Vec::from_iter(1..=3);
//         let quorums_operators = mocked_state_retriever
//             .getOperatorState(
//                 *mocked_registry_coordinator.address(),
//                 quorum_numbers.into(),
//                 1,
//             )
//             .call()
//             .await
//             .unwrap()
//             ._0;
//         assert_eq!(quorums_operators.len(), 3);

//         let eigen_client = EigenStakingClient::new(
//             anvil_provider,
//             *mocked_registry_coordinator.address(),
//             Some(*mocked_state_retriever.address()),
//         );
//         let avs_operators = eigen_client.get_avs_operators(1).await.unwrap();
//         let count = avs_operators.get_quorum_count();
//         assert_eq!(count, 3);
//         let operator_set = avs_operators.get_operator_set();
//         assert_eq!(operator_set.len(), 2);
//         assert!(operator_set.contains(&Operator {
//             address: operator_1_quorum_1.operator,
//             id: operator_1_quorum_1.operatorId,
//         }));
//         assert!(operator_set.contains(&Operator {
//             address: operator_2_quorum_1.operator,
//             id: operator_2_quorum_1.operatorId,
//         }));
//     }

//     fn generate_operator() -> OperatorStateRetriever::Operator {
//         let mut rng = rand::thread_rng();
//         let stake = Uint::<96, 2>::from(rng.gen::<u64>());
//         let mut id = [0u8; 32];
//         let mut address = [0u8; 20];
//         rng.fill(&mut id);
//         rng.fill(&mut address);
//         OperatorStateRetriever::Operator {
//             operator: Address::from(address),
//             operatorId: FixedBytes::from(id),
//             stake,
//         }
//     }

//     fn update_operator_stake(
//         operator: &OperatorStateRetriever::Operator,
//     ) -> OperatorStateRetriever::Operator {
//         let mut rng = rand::thread_rng();
//         let stake = Uint::<96, 2>::from(rng.gen::<u64>());
//         OperatorStateRetriever::Operator {
//             operator: operator.operator,
//             operatorId: operator.operatorId,
//             stake,
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the AVS registry service
    let _avs_registry_service = init_avs_registry_service().await;
    
    // The service is now initialized and ready to use
    println!("AVS Registry Service initialized successfully!");
    Ok(())
}
