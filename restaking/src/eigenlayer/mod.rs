use alloy_primitives::{address, Address, FixedBytes, Uint, U256, Bytes};
use alloy_network::Network;
use alloy::providers::RootProvider;
use alloy_sol_types::sol
use alloy_transport::Transport;
use alloy_contract::Error as ContractError;
use eigen_utils::middleware::{operatorstateretriever::OperatorStateRetriever, registrycoordinator::RegistryCoordinator};
use url::Url;
use std::{collections::HashSet, time::Duration};
use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_logging::{get_logger, init_logger};
use eigen_logging::logger::Logger;
use eigen_logging::noop_logger::NoopLogger;
use eigen_services_avsregistry::chaincaller::AvsRegistryServiceChainCaller;
use eigen_services_operatorsinfo::operatorsinfo_inmemory::OperatorInfoServiceInMemory;
use eigen_crypto_bls::{Signature, BlsG2Point};
use eigen_types::{
    avs::TaskResponseDigest,
    operator::QuorumThresholdPercentages,
};


// Codegen from ABI file to interact with the OperatorStateRetriever contract.
sol!(
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    OperationStateRetriever,
    "src/eigenlayer/abi/operator_state_retriever.json"
);

// Codegen from ABI file to interact with the RegistryCoordinator contract.
sol!(
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    RegistryCoordinator,
    "src/eigenlayer/abi/registry_coordinator.json"
);

/// source: https://github.com/Layr-Labs/eigenlayer-middleware 
/// OperatorStateRetriever is compatible with Eigenlayer's BLS middleware, 
/// this is the address of the OperatorStateRetriever contract deployed Ethereum mainnet.
const OPERATOR_STATE_RETRIEVER_ADDRESS: Address =
    address!("0xd5d7fb4647ce79740e6e83819efdf43fa74f8c31");

pub struct EigenStakingClient<T: Transport + std::clone::Clone, N: alloy_network::Network> {
    provider: RootProvider<T, N>,
    operator_state_retriever_address: Address,
    registry_coordinator_address: Address,
}

impl<T: Transport + std::clone::Clone, N: alloy_network::Network> EigenStakingClient<T, N> {
    pub async fn new(
        provider: RootProvider<T, N>,
        registry_coordinator_address: Address,
        operator_state_retriever_address: Option<Address>,
    ) -> Self {
        let registry_coordinator_address: Address = address!("eCd099fA5048c3738a5544347D8cBc8076E76494").into(); // TODO: get from config
        let operator_state_retriever_address: Address = address!("D5D7fB4647cE79740E6e83819EFDf43fa74F8C31").into(); // TODO: get from config
        let http_endpoint = String::from("http://ethereum:8545"); // TODO: get from .env
        let ws_endpoint = String::from("ws://ethereum:8545"); // TODO: get from .env
    
        let _provider: RootProvider<_, Ethereum> = RootProvider::new_http(Url::parse(&http_endpoint).unwrap());
        let current_block_num = _provider.get_block_number().await.unwrap();
        let quorum_nums = Bytes::from([0u8]);
        let quorum_threshold_percentages: QuorumThresholdPercentages = vec![33];
        let time_to_expiry = Duration::from_secs(1000);
    
        // Create avs clients to interact with contracts deployed on anvil
        let avs_registry_reader = AvsRegistryChainReader::new(
            get_logger().clone(),
            registry_coordinator_address,
            operator_state_retriever_address,
            http_endpoint.clone(),
        ).await.unwrap();
    
        let (operators_info, _rx) = OperatorInfoServiceInMemory::new(
            get_logger(),
            avs_registry_reader.clone(),
            ws_endpoint,
        ).await.unwrap();
        let operators_info_clone = operators_info.clone();
    
        Self {
            provider,
            registry_coordinator_address,
            operator_state_retriever_address,
        }
    }

    pub async fn get_avs_operators(
        &self,
        block_number: u32,
    ) -> Result<OperatorState, ContractError> {
        let registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_address, self.provider.clone());
        let operator_state_retriever = OperatorStateRetriever::new(
            self.operator_state_retriever_address,
            self.provider.clone(),
        );

        let builder = registry_coordinator.quorumCount();
        let quorum_count = builder.call().await?._0;
        let quorum_numbers: Vec<u8> = Vec::from_iter(1..=quorum_count);
        let operators_state = operator_state_retriever
            .getOperatorState_0(
                self.registry_coordinator_address,
                quorum_numbers.into(),
                block_number,
            )
            .call()
            .await?
            ._0;
        Ok(OperatorState::new(block_number, operators_state))
    }
}

pub struct OperatorState {
    block_number: u32,
    quorums_operators: Vec<Vec<OperatorStateRetriever::Operator>>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Operator {
    address: Address,
    id: FixedBytes<32>,
}

impl OperatorState {
    fn new(
        block_number: u32,
        quorums_operators: Vec<Vec<OperatorStateRetriever::Operator>>,
    ) -> Self {
        Self {
            block_number,
            quorums_operators,
        }
    }

    pub fn get_block_number(&self) -> u32 {
        self.block_number
    }

    pub fn get_quorum_count(&self) -> usize {
        self.quorums_operators.len()
    }

    pub fn get_operator_set(&self) -> HashSet<Operator> {
        let mut set = HashSet::new();
        for quorum_operator_list in &self.quorums_operators {
            for operator in quorum_operator_list {
                set.insert(Operator {
                    address: operator.operator,
                    id: operator.operatorId,
                });
            }
        }
        set
    }

    /// Returns the (OperatorStake,TotalStake) of the provided quorum number.
    pub fn get_operator_weight(
        &self,
        operator_id: FixedBytes<32>,
        quorum_number: usize,
    ) -> Option<(U256, U256)> {
        let quorum_operators = self.quorums_operators.get(quorum_number - 1)?;
        let mut operator_staked: Option<U256> = None;
        let mut total_staked: U256 = Uint::from(0);
        for operator in quorum_operators {
            let stake = operator.stake;
            total_staked = total_staked.saturating_add(U256::from(stake));
            if operator_id == operator.operatorId {
                operator_staked = Some(U256::from(stake));
            }
        }
        Some((operator_staked?, total_staked))
    }
}

#[cfg(test)]
mod tests {
    use alloy::sol_types::private;
    use alloy::{providers::ProviderBuilder, sol};
    use alloy_node_bindings::Anvil;

    use super::*;
    use rand::Rng;
    use std::sync::Arc;

    sol!(
        #[allow(missing_docs)]
        #[sol(rpc)]
        MockedRegistryCoordinator,
        "src/eigenlayer/artifacts/registry_coordinator.sol/RegistryCoordinator.json"
    );
    sol!(
        #[allow(missing_docs)]
        #[sol(rpc)]
        MockedOperatorStateRetriever,
        "src/eigenlayer/artifacts/operator_state_retriever.sol/OperatorStateRetriever.json"
    );

    #[tokio::test]
    async fn test_mocked_registry_coordinator() {
        let anvil = Anvil::new().block_time(1_u64).spawn();
        let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());
        let anvil_provider = Arc::new(anvil_provider);
        let coordinator = MockedRegistryCoordinator::deploy(anvil_provider)
            .await
            .unwrap();
        coordinator
            .setQuorumCount(3)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();
        let count = coordinator.quorumCount().call().await.unwrap()._0;
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_mocked_operator_state_retriever() {
        let anvil = Anvil::new().block_time(1_u64).spawn();

        let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());
        let anvil_provider = Arc::new(anvil_provider);

        let coordinator = MockedRegistryCoordinator::deploy(anvil_provider.clone())
            .await
            .unwrap();

        let retriever = MockedOperatorStateRetriever::deploy(anvil_provider)
            .await
            .unwrap();
        retriever
            .setRegistryCoordinator(*coordinator.address())
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        let registry_coordinator_address =
            retriever._registryCoordinator().call().await.unwrap()._0;
        assert_eq!(&registry_coordinator_address, coordinator.address());
    }

    #[tokio::test]
    async fn test_eigen_layer_client() {
        let anvil = Anvil::new().block_time(1_u64).spawn();

        let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());

        let mocked_registry_coordinator = MockedRegistryCoordinator::deploy(anvil_provider.clone())
            .await
            .unwrap();

        let mocked_state_retriever = MockedOperatorStateRetriever::deploy(anvil_provider.clone())
            .await
            .unwrap();

        let _ = mocked_registry_coordinator
            .setQuorumCount(3)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();
        let count = mocked_registry_coordinator
            .quorumCount()
            .call()
            .await
            .unwrap()
            ._0;
        assert_eq!(count, 3);

        let operator_1_quorum_1 = generate_operator();
        let operator_2_quorum_1 = generate_operator();
        let operator_1_quorum_3 = update_operator_stake(&operator_1_quorum_1);
        let operator_2_quorum_3 = update_operator_stake(&operator_2_quorum_1);

        let operators_quorum_1: private::Vec<OperatorStateRetriever::Operator> =
            vec![operator_1_quorum_1.clone(), operator_2_quorum_1.clone()];
        let operators_quorum_2: private::Vec<OperatorStateRetriever::Operator> = vec![];
        let operators_quorum_3: private::Vec<OperatorStateRetriever::Operator> =
            vec![operator_1_quorum_3, operator_2_quorum_3];

        mocked_state_retriever
            .setOperators(1, operators_quorum_1)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();
        mocked_state_retriever
            .setOperators(2, operators_quorum_2)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();
        mocked_state_retriever
            .setOperators(3, operators_quorum_3)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        let quorum_numbers: Vec<u8> = Vec::from_iter(1..=3);
        let quorums_operators = mocked_state_retriever
            .getOperatorState(
                *mocked_registry_coordinator.address(),
                quorum_numbers.into(),
                1,
            )
            .call()
            .await
            .unwrap()
            ._0;
        assert_eq!(quorums_operators.len(), 3);

        let eigen_client = EigenStakingClient::new(
            anvil_provider,
            *mocked_registry_coordinator.address(),
            Some(*mocked_state_retriever.address()),
        );
        let avs_operators = eigen_client.get_avs_operators(1).await.unwrap();
        let count = avs_operators.get_quorum_count();
        assert_eq!(count, 3);
        let operator_set = avs_operators.get_operator_set();
        assert_eq!(operator_set.len(), 2);
        assert!(operator_set.contains(&Operator {
            address: operator_1_quorum_1.operator,
            id: operator_1_quorum_1.operatorId,
        }));
        assert!(operator_set.contains(&Operator {
            address: operator_2_quorum_1.operator,
            id: operator_2_quorum_1.operatorId,
        }));
    }

    fn generate_operator() -> OperatorStateRetriever::Operator {
        let mut rng = rand::thread_rng();
        let stake = Uint::<96, 2>::from(rng.gen::<u64>());
        let mut id = [0u8; 32];
        let mut address = [0u8; 20];
        rng.fill(&mut id);
        rng.fill(&mut address);
        OperatorStateRetriever::Operator {
            operator: Address::from(address),
            operatorId: FixedBytes::from(id),
            stake,
        }
    }

    fn update_operator_stake(
        operator: &OperatorStateRetriever::Operator,
    ) -> OperatorStateRetriever::Operator {
        let mut rng = rand::thread_rng();
        let stake = Uint::<96, 2>::from(rng.gen::<u64>());
        OperatorStateRetriever::Operator {
            operator: operator.operator,
            operatorId: operator.operatorId,
            stake,
        }
    }
}
