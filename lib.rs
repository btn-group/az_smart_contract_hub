#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;
mod validations;

#[ink::contract]
mod az_smart_contract_hub {
    use crate::errors::{AZGroupsError, AZSmartContractHubError};
    use crate::validations::validate_presence_of;
    use ink::prelude::string::{String, ToString};
    use ink::storage::Mapping;

    // === EVENTS ===
    #[ink(event)]
    pub struct Create {
        #[ink(topic)]
        id: u32,
        #[ink(topic)]
        smart_contract_address: AccountId,
        environment: u8,
        #[ink(topic)]
        caller: AccountId,
        azero_id_domain: String,
        link_to_abi: String,
        link_to_contract: Option<String>,
        link_to_wasm: Option<String>,
        link_to_audit: Option<String>,
        group_id: Option<u32>,
        project_name: Option<String>,
        project_website: Option<String>,
        github: Option<String>,
    }

    #[ink(event)]
    pub struct Update {
        #[ink(topic)]
        id: u32,
        enabled: bool,
        azero_id_domain: String,
        link_to_audit: Option<String>,
        group_id: Option<u32>,
        project_name: Option<String>,
        project_website: Option<String>,
        github: Option<String>,
    }

    // === STRUCTS ===
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Config {
        az_groups_address: AccountId,
        azero_id_router_address: AccountId,
        smart_contracts_count: u32,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    struct GroupUser {
        role: u8,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SmartContract {
        id: u32,
        smart_contract_address: AccountId,
        environment: u8,
        caller: AccountId,
        enabled: bool,
        azero_id_domain: String,
        link_to_abi: String,
        link_to_contract: Option<String>,
        link_to_wasm: Option<String>,
        link_to_audit: Option<String>,
        group_id: Option<u32>,
        project_name: Option<String>,
        project_website: Option<String>,
        github: Option<String>,
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
        pub fn config(&self) -> Config {
            Config {
                az_groups_address: self.az_groups_address,
                azero_id_router_address: self.azero_id_router_address,
                smart_contracts_count: self.smart_contracts_count,
            }
        }

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
        #[allow(clippy::too_many_arguments)]
        #[ink(message)]
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            environment: u8,
            azero_id_domain: String,
            link_to_abi: String,
            link_to_contract: Option<String>,
            link_to_wasm: Option<String>,
            link_to_audit: Option<String>,
            group_id: Option<u32>,
            project_name: Option<String>,
            project_website: Option<String>,
            github: Option<String>,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            if self.smart_contracts_count == u32::MAX {
                return Err(AZSmartContractHubError::UnprocessableEntity(
                    "Smart contract limit reached".to_string(),
                ));
            }
            let caller: AccountId = Self::env().caller();
            if caller != self.address_by_domain(azero_id_domain.clone())? {
                return Err(AZSmartContractHubError::Unauthorised);
            }
            if let Some(group_id_unwrapped) = group_id {
                let group_user: GroupUser = self.group_users_show(group_id_unwrapped, caller)?;
                if group_user.role == 0 {
                    return Err(AZSmartContractHubError::Unauthorised);
                }
            }
            let link_to_abi_formatted: String = self.format_url(link_to_abi);
            validate_presence_of(&link_to_abi_formatted, "Link to abi")?;

            let smart_contract: SmartContract = SmartContract {
                id: self.smart_contracts_count,
                smart_contract_address,
                environment,
                caller: Self::env().caller(),
                enabled: true,
                azero_id_domain: azero_id_domain.clone(),
                group_id,
                link_to_abi: link_to_abi_formatted.clone(),
                link_to_contract: link_to_contract.clone(),
                link_to_wasm: link_to_wasm.clone(),
                link_to_audit: link_to_audit.clone(),
                project_name: project_name.clone(),
                project_website: project_website.clone(),
                github: github.clone(),
            };
            self.smart_contracts
                .insert(self.smart_contracts_count, &smart_contract);
            self.smart_contracts_count += 1;

            // emit event
            self.env().emit_event(Create {
                id: smart_contract.id,
                smart_contract_address,
                environment,
                caller,
                azero_id_domain,
                link_to_abi: link_to_abi_formatted,
                link_to_contract,
                link_to_wasm,
                link_to_audit,
                group_id,
                project_name,
                project_website,
                github,
            });

            Ok(smart_contract)
        }

        #[allow(clippy::too_many_arguments)]
        #[ink(message)]
        pub fn update(
            &mut self,
            id: u32,
            enabled: bool,
            azero_id_domain: String,
            group_id: Option<u32>,
            link_to_audit: Option<String>,
            project_name: Option<String>,
            project_website: Option<String>,
            github: Option<String>,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            let mut smart_contract: SmartContract = self.show(id)?;
            let caller: AccountId = Self::env().caller();
            if caller != smart_contract.caller {
                return Err(AZSmartContractHubError::Unauthorised);
            }

            smart_contract.enabled = enabled;
            if smart_contract.azero_id_domain != azero_id_domain {
                if caller != self.address_by_domain(azero_id_domain.clone())? {
                    return Err(AZSmartContractHubError::Unauthorised);
                }
                smart_contract.azero_id_domain = azero_id_domain.clone()
            };
            if let Some(group_id_unwrapped) = group_id {
                let group_user: GroupUser = self.group_users_show(group_id_unwrapped, caller)?;
                if group_user.role == 0 {
                    return Err(AZSmartContractHubError::Unauthorised);
                }
            };
            smart_contract.group_id = group_id;
            smart_contract.project_name = project_name.clone();
            smart_contract.project_website = project_website.clone();
            smart_contract.github = github.clone();
            smart_contract.link_to_audit = link_to_audit.clone();
            self.smart_contracts
                .insert(smart_contract.id, &smart_contract);

            // emit event
            self.env().emit_event(Update {
                id: smart_contract.id,
                enabled: smart_contract.enabled,
                azero_id_domain,
                group_id,
                project_name,
                project_website,
                github,
                link_to_audit,
            });

            Ok(smart_contract)
        }

        fn address_by_domain(&self, domain: String) -> Result<AccountId, AZSmartContractHubError> {
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

        fn format_url(&self, url: String) -> String {
            url.trim().to_string()
        }

        fn group_users_show(
            &self,
            group_id: u32,
            user: AccountId,
        ) -> Result<GroupUser, AZSmartContractHubError> {
            match cfg!(test) {
                true => unimplemented!(
                    "`invoke_contract()` not being supported (tests end up panicking)"
                ),
                false => {
                    use ink::env::call::{build_call, ExecutionInput, Selector};

                    const GROUP_USERS_SHOW_SELECTOR: [u8; 4] =
                        ink::selector_bytes!("group_users_show");
                    Ok(build_call::<Environment>()
                        .call(self.az_groups_address)
                        .exec_input(
                            ExecutionInput::new(Selector::new(GROUP_USERS_SHOW_SELECTOR))
                                .push_arg(group_id)
                                .push_arg(user),
                        )
                        .returns::<core::result::Result<GroupUser, AZGroupsError>>()
                        .invoke()?)
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        // const MOCK_AZERO_ID: &str = "OnionKnight";
        // const MOCK_LINK_TO_ABI: &str = "https://res.cloudinary.com/xasdf123/raw/upload/v1690808298/smart_contract_metadata/tmuurccd5a7lcvin6ae9.json";

        // === HELPERS ===
        fn init() -> (DefaultAccounts<DefaultEnvironment>, AZSmartContractHub) {
            let accounts = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.bob);
            let az_smart_contract_hub = AZSmartContractHub::new(accounts.eve, accounts.frank);
            (accounts, az_smart_contract_hub)
        }

        // === TESTS ===
        // === TEST QUERIES ===
        #[ink::test]
        fn test_config() {
            let (accounts, az_smart_contract_hub) = init();
            let config = az_smart_contract_hub.config();
            // * it returns the config
            assert_eq!(config.azero_id_router_address, accounts.eve);
            assert_eq!(config.az_groups_address, accounts.frank);
            assert_eq!(config.smart_contracts_count, 0);
        }

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
        //             MOCK_LINK_TO_ABI.to_string(),
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
        //         MOCK_LINK_TO_ABI.to_string(),
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
        // fn test_update() {
        //     let (accounts, mut az_smart_contract_hub) = init();
        //     // = when smart_contract doesn't exist
        //     // = * it raises an error
        //     let mut result = az_smart_contract_hub.update(0, false);
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
        //             MOCK_LINK_TO_ABI.to_string(),
        //             0,
        //             MOCK_AZERO_ID.to_string(),
        //         )
        //         .unwrap();
        //     // == when called by non-caller
        //     set_caller::<DefaultEnvironment>(accounts.charlie);
        //     // == * it raises an error
        //     result = az_smart_contract_hub.update(0, false);
        //     assert_eq!(result, Err(AZSmartContractHubError::Unauthorised));
        //     // == when called by caller
        //     set_caller::<DefaultEnvironment>(accounts.bob);
        //     // === when smart_contract is already enabled
        //     // ==== when the user tries to enable
        //     // ==== * it raises an error
        //     result = az_smart_contract_hub.update(0, true);
        //     assert_eq!(
        //         result,
        //         Err(AZSmartContractHubError::Unchanged("Enabled".to_string()))
        //     );
        //     // ==== when the user tries to disable
        //     // ==== * it updates the smart_contract enabled to false
        //     result = az_smart_contract_hub.update(0, false);
        //     assert_eq!(result.unwrap().enabled, false);

        //     // === when smart_contract is already disabled
        //     // ==== when the user tries to disable
        //     // ==== * it raises an error
        //     result = az_smart_contract_hub.update(0, false);
        //     assert_eq!(
        //         result,
        //         Err(AZSmartContractHubError::Unchanged("Enabled".to_string()))
        //     );
        //     // ==== when the user tries to enable
        //     // ==== * it updates the smart_contract enabled to true
        //     result = az_smart_contract_hub.update(0, true);
        //     assert_eq!(result.unwrap().enabled, true);
        // }
    }
}
