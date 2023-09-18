#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_smart_contract_hub {
    use ink::prelude::string::{String, ToString};
    use ink::storage::Mapping;

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AZSmartContractHubError {
        NotFound(String),
        Unauthorised,
        Unchanged(String),
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct Create {
        #[ink(topic)]
        id: u32,
        #[ink(topic)]
        smart_contract_address: AccountId,
        url: String,
        environment: u8,
        #[ink(topic)]
        caller: AccountId,
    }

    #[ink(event)]
    pub struct Toggle {
        #[ink(topic)]
        id: u32,
        enabled: bool,
    }

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SmartContract {
        id: u32,
        smart_contract_address: AccountId,
        url: String,
        environment: u8,
        caller: AccountId,
        enabled: bool,
    }

    // === CONTRACT ===
    #[ink(storage)]
    pub struct AZSmartContractHub {
        az_groups_address: AccountId,
        azero_id_router_address: AccountId,
        smart_contracts: Mapping<u32, SmartContract>,
        smart_contracts_count: u32,
    }
    impl AZSmartContractHub {
        #[ink(constructor)]
        pub fn new(azero_id_router_address: AccountId, az_groups_address: AccountId) -> Self {
            Self {
                az_groups_address,
                azero_id_router_address,
                smart_contracts: Mapping::default(),
                smart_contracts_count: 0,
            }
        }

        // === QUERIES ===
        #[ink(message)]
        pub fn show(&self, id: u32) -> Result<SmartContract, AZSmartContractHubError> {
            if let Some(smart_contract) = self.smart_contracts.get(id) {
                Ok(smart_contract)
            } else {
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string(),
                ))
            }
        }

        // === HANDLES ===
        // 0 == Production
        // 1 == Testnet
        // 2 == Smarknet
        #[ink(message)]
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            url: String,
            environment: u8,
            azero_id_domain: String,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            let caller: AccountId = Self::env().caller();
            if caller != self.get_address_by_domain(azero_id_domain)? {
                return Err(AZSmartContractHubError::Unauthorised);
            }

            let smart_contract: SmartContract = SmartContract {
                id: self.smart_contracts_count,
                smart_contract_address,
                url: url.clone(),
                environment,
                caller: Self::env().caller(),
                enabled: true,
            };
            self.smart_contracts
                .insert(self.smart_contracts_count, &smart_contract);
            self.smart_contracts_count += 1;

            // emit event
            self.env().emit_event(Create {
                id: smart_contract.id,
                smart_contract_address,
                url,
                environment,
                caller,
            });

            Ok(smart_contract)
        }

        #[ink(message)]
        pub fn toggle_enabled(
            &mut self,
            id: u32,
            enabled: bool,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            let mut smart_contract: SmartContract = self.show(id)?;
            let caller: AccountId = Self::env().caller();
            if caller != smart_contract.caller {
                return Err(AZSmartContractHubError::Unauthorised);
            }
            if smart_contract.enabled == enabled {
                return Err(AZSmartContractHubError::Unchanged("Enabled".to_string()));
            }

            smart_contract.enabled = enabled;
            self.smart_contracts
                .insert(smart_contract.id, &smart_contract);

            // emit event
            self.env().emit_event(Toggle {
                id: smart_contract.id,
                enabled: smart_contract.enabled,
            });

            Ok(smart_contract)
        }

        fn get_address_by_domain(
            &self,
            domain: String,
        ) -> Result<AccountId, AZSmartContractHubError> {
            match cfg!(test) {
                true => unimplemented!(
                    "`invoke_contract()` not being supported (tests end up panicking)"
                ),
                false => {
                    use ink::env::call::{build_call, ExecutionInput, Selector};
                    // const GET_ADDRESS_SELECTOR: [u8; 4] = [0xD2, 0x59, 0xF7, 0xBA];
                    const GET_ADDRESS_SELECTOR: [u8; 4] = ink::selector_bytes!("get_address");
                    let result = build_call::<Environment>()
                        .call(self.azero_id_router_address)
                        .exec_input(
                            ExecutionInput::new(Selector::new(GET_ADDRESS_SELECTOR))
                                .push_arg(domain),
                        )
                        .returns::<core::result::Result<AccountId, u8>>()
                        .params()
                        .invoke();
                    // Use the result as per the need
                    if let Ok(address) = result {
                        Ok(address)
                    } else {
                        Err(AZSmartContractHubError::NotFound("Domain".to_string()))
                    }
                }
            }
        }
    }

    // #[cfg(test)]
    // mod tests {
    //     use super::*;
    //     use ink::env::{
    //         test::{default_accounts, set_caller, DefaultAccounts},
    //         DefaultEnvironment,
    //     };

    // const MOCK_AZERO_ID: &str = "OnionKnight";
    // const MOCK_URL: &str = "https://res.cloudinary.com/xasdf123/raw/upload/v1690808298/smart_contract_metadata/tmuurccd5a7lcvin6ae9.json";

    // === HELPERS ===
    // fn init() -> (DefaultAccounts<DefaultEnvironment>, AZSmartContractHub) {
    //     let accounts = default_accounts();
    //     set_caller::<DefaultEnvironment>(accounts.bob);
    //     let az_smart_contract_hub = AZSmartContractHub::new(accounts.frank);
    //     (accounts, az_smart_contract_hub)
    // }

    // === TESTS ===
    // === TEST QUERIES ===
    // #[ink::test]
    // fn test_show() {
    //     let (accounts, mut az_smart_contract_hub) = init();
    //     // = when smart_contract does not exist
    //     // * it returns error
    //     assert_eq!(
    //         az_smart_contract_hub.show(0),
    //         Err(AZSmartContractHubError::NotFound(
    //             "SmartContract".to_string()
    //         ))
    //     );
    //     // = when smart_contract exists
    //     let smart_contract: SmartContract = az_smart_contract_hub
    //         .create(
    //             accounts.alice,
    //             MOCK_URL.to_string(),
    //             0,
    //             MOCK_AZERO_ID.to_string(),
    //         )
    //         .unwrap();
    //     // = * it returns the smart_contract
    //     assert_eq!(
    //         az_smart_contract_hub.show(smart_contract.id),
    //         Ok(smart_contract)
    //     );
    // }

    // === TEST HANDLES ===
    // #[ink::test]
    // fn test_create() {
    //     let (accounts, mut az_smart_contract_hub) = init();
    //     // when environment is within range
    //     az_smart_contract_hub.smart_contracts_count = u32::MAX - 1;
    //     // * it stores the caller as the caller
    //     let result = az_smart_contract_hub.create(
    //         accounts.alice,
    //         MOCK_URL.to_string(),
    //         0,
    //         MOCK_AZERO_ID.to_string(),
    //     );
    //     let result_unwrapped = result.unwrap();
    //     // * it stores the id as the current length
    //     assert_eq!(result_unwrapped.id, u32::MAX - 1);
    //     // * it increases the smart_contracts length by 1
    //     assert_eq!(az_smart_contract_hub.smart_contracts_count, u32::MAX);
    //     // * it stores the submitted smart contract address
    //     assert_eq!(result_unwrapped.smart_contract_address, accounts.alice);
    //     // * it sets the caller to the caller
    //     assert_eq!(result_unwrapped.caller, accounts.bob);
    //     // * it stores the smart_contract
    //     assert_eq!(
    //         result_unwrapped,
    //         az_smart_contract_hub.show(result_unwrapped.id).unwrap()
    //     );
    // }

    // #[ink::test]
    // fn test_toggle_enabled() {
    //     let (accounts, mut az_smart_contract_hub) = init();
    //     // = when smart_contract doesn't exist
    //     // = * it raises an error
    //     let mut result = az_smart_contract_hub.toggle_enabled(0, false);
    //     assert_eq!(
    //         result,
    //         Err(AZSmartContractHubError::NotFound(
    //             "SmartContract".to_string()
    //         ))
    //     );

    //     // = when smart_contract exists
    //     az_smart_contract_hub
    //         .create(
    //             accounts.alice,
    //             MOCK_URL.to_string(),
    //             0,
    //             MOCK_AZERO_ID.to_string(),
    //         )
    //         .unwrap();
    //     // == when called by non-caller
    //     set_caller::<DefaultEnvironment>(accounts.charlie);
    //     // == * it raises an error
    //     result = az_smart_contract_hub.toggle_enabled(0, false);
    //     assert_eq!(result, Err(AZSmartContractHubError::Unauthorised));
    //     // == when called by caller
    //     set_caller::<DefaultEnvironment>(accounts.bob);
    //     // === when smart_contract is already enabled
    //     // ==== when the user tries to enable
    //     // ==== * it raises an error
    //     result = az_smart_contract_hub.toggle_enabled(0, true);
    //     assert_eq!(
    //         result,
    //         Err(AZSmartContractHubError::Unchanged("Enabled".to_string()))
    //     );
    //     // ==== when the user tries to disable
    //     // ==== * it updates the smart_contract enabled to false
    //     result = az_smart_contract_hub.toggle_enabled(0, false);
    //     assert_eq!(result.unwrap().enabled, false);

    //     // === when smart_contract is already disabled
    //     // ==== when the user tries to disable
    //     // ==== * it raises an error
    //     result = az_smart_contract_hub.toggle_enabled(0, false);
    //     assert_eq!(
    //         result,
    //         Err(AZSmartContractHubError::Unchanged("Enabled".to_string()))
    //     );
    //     // ==== when the user tries to enable
    //     // ==== * it updates the smart_contract enabled to true
    //     result = az_smart_contract_hub.toggle_enabled(0, true);
    //     assert_eq!(result.unwrap().enabled, true);
    // }
    // }
}
