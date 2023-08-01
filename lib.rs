#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_smart_contract_metadata_hub {
    #[ink(storage)]
    pub struct AzSmartContractMetadataHub {
        value: bool,
    }

    impl AzSmartContractMetadataHub {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let az_smart_contract_metadata_hub = AzSmartContractMetadataHub::default();
            assert_eq!(az_smart_contract_metadata_hub.get(), false);
        }

        #[ink::test]
        fn it_works() {
            let mut az_smart_contract_metadata_hub = AzSmartContractMetadataHub::new(false);
            assert_eq!(az_smart_contract_metadata_hub.get(), false);
            az_smart_contract_metadata_hub.flip();
            assert_eq!(az_smart_contract_metadata_hub.get(), true);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = AzSmartContractMetadataHubRef::default();

            // When
            let contract_account_id = client
                .instantiate("az_smart_contract_metadata_hub", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<AzSmartContractMetadataHubRef>(contract_account_id.clone())
                .call(|az_smart_contract_metadata_hub| az_smart_contract_metadata_hub.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = AzSmartContractMetadataHubRef::new(false);
            let contract_account_id = client
                .instantiate("az_smart_contract_metadata_hub", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<AzSmartContractMetadataHubRef>(contract_account_id.clone())
                .call(|az_smart_contract_metadata_hub| az_smart_contract_metadata_hub.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<AzSmartContractMetadataHubRef>(contract_account_id.clone())
                .call(|az_smart_contract_metadata_hub| az_smart_contract_metadata_hub.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<AzSmartContractMetadataHubRef>(contract_account_id.clone())
                .call(|az_smart_contract_metadata_hub| az_smart_contract_metadata_hub.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
