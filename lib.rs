#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;

#[ink::contract]
mod az_smart_contract_hub {
    use crate::errors::{AZGroupsError, AZSmartContractHubError};
    use ink::{
        codegen::EmitEvent,
        env::call::{build_call, ExecutionInput, Selector},
        prelude::{
            format,
            string::{String, ToString},
        },
        reflect::ContractEventBase,
        storage::Mapping,
    };

    const MOCK_VALID_AZERO_ID: &str = "MOCK VALID AZERO ID";
    const MOCK_INVALID_AZERO_ID: &str = "MOCK INVALID AZERO ID";

    // === TYPES ===
    type Event = <AZSmartContractHub as ContractEventBase>::Type;
    type Result<T> = core::result::Result<T, AZSmartContractHubError>;

    // === ENUMS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    pub enum Role {
        Banned,
        Applicant,
        Member,
        Admin,
        SuperAdmin,
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct Create {
        #[ink(topic)]
        id: u32,
        #[ink(topic)]
        smart_contract_address: AccountId,
        chain: u8,
        #[ink(topic)]
        caller: AccountId,
        azero_id: String,
        abi_url: String,
        contract_url: Option<String>,
        wasm_url: Option<String>,
        audit_url: Option<String>,
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
        azero_id: String,
        audit_url: Option<String>,
        group_id: Option<u32>,
        project_name: Option<String>,
        project_website: Option<String>,
        github: Option<String>,
    }

    // === STRUCTS ===
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Config {
        admin: AccountId,
        az_groups_address: AccountId,
        azero_id_router_address: AccountId,
        fee: Balance,
        smart_contracts_count: u32,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SmartContract {
        id: u32,
        smart_contract_address: AccountId,
        chain: u8,
        caller: AccountId,
        enabled: bool,
        azero_id: String,
        abi_url: String,
        contract_url: Option<String>,
        wasm_url: Option<String>,
        audit_url: Option<String>,
        group_id: Option<u32>,
        project_name: Option<String>,
        project_website: Option<String>,
        github: Option<String>,
    }

    // === CONTRACT ===
    #[ink(storage)]
    pub struct AZSmartContractHub {
        admin: AccountId,
        az_groups_address: AccountId,
        azero_id_router_address: AccountId,
        fee: Balance,
        smart_contracts: Mapping<u32, SmartContract>,
        smart_contracts_count: u32,
    }
    impl AZSmartContractHub {
        #[ink(constructor)]
        pub fn new(azero_id_router_address: AccountId, az_groups_address: AccountId) -> Self {
            Self {
                admin: Self::env().caller(),
                az_groups_address,
                azero_id_router_address,
                fee: 1_000,
                smart_contracts: Mapping::default(),
                smart_contracts_count: 0,
            }
        }

        // === QUERIES ===
        #[ink(message)]
        pub fn config(&self) -> Config {
            Config {
                admin: self.admin,
                az_groups_address: self.az_groups_address,
                azero_id_router_address: self.azero_id_router_address,
                fee: self.fee,
                smart_contracts_count: self.smart_contracts_count,
            }
        }

        #[ink(message)]
        pub fn show(&self, id: u32) -> Result<SmartContract> {
            self.smart_contracts
                .get(id)
                .ok_or(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string(),
                ))
        }

        // === HANDLES ===
        #[allow(clippy::too_many_arguments)]
        #[ink(message, payable)]
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            chain: u8,
            azero_id: String,
            abi_url: String,
            contract_url: Option<String>,
            wasm_url: Option<String>,
            audit_url: Option<String>,
            group_id: Option<u32>,
            project_name: Option<String>,
            project_website: Option<String>,
            github: Option<String>,
        ) -> Result<SmartContract> {
            if self.smart_contracts_count == u32::MAX {
                return Err(AZSmartContractHubError::UnprocessableEntity(
                    "Smart contract limit reached".to_string(),
                ));
            }
            let caller: AccountId = Self::env().caller();
            self.validate_ownership_of_azero_id(azero_id.clone(), caller)?;
            if let Some(group_id_unwrapped) = group_id {
                self.validate_membership(group_id_unwrapped, caller)?;
            }
            let abi_url_formatted: String = self.format_url(abi_url);
            Self::validate_presence_of(&abi_url_formatted, "Link to abi")?;
            if self.env().transferred_value() != self.fee {
                return Err(AZSmartContractHubError::UnprocessableEntity(
                    "Incorrect fee".to_string(),
                ));
            }

            let smart_contract: SmartContract = SmartContract {
                id: self.smart_contracts_count,
                smart_contract_address,
                chain,
                caller: Self::env().caller(),
                enabled: true,
                azero_id: azero_id.clone(),
                group_id,
                abi_url: abi_url_formatted.clone(),
                contract_url: contract_url.clone(),
                wasm_url: wasm_url.clone(),
                audit_url: audit_url.clone(),
                project_name: project_name.clone(),
                project_website: project_website.clone(),
                github: github.clone(),
            };
            self.smart_contracts
                .insert(self.smart_contracts_count, &smart_contract);
            self.smart_contracts_count = self.smart_contracts_count.checked_add(1).unwrap();

            // Transfer fee to admin
            if self.env().transfer(self.admin, self.fee).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }

            // emit event
            Self::emit_event(
                self.env(),
                Event::Create(Create {
                    id: smart_contract.id,
                    smart_contract_address,
                    chain,
                    caller,
                    azero_id,
                    abi_url: abi_url_formatted,
                    contract_url,
                    wasm_url,
                    audit_url,
                    group_id,
                    project_name,
                    project_website,
                    github,
                }),
            );

            Ok(smart_contract)
        }

        #[allow(clippy::too_many_arguments)]
        #[ink(message)]
        pub fn update(
            &mut self,
            id: u32,
            enabled: bool,
            azero_id: String,
            group_id: Option<u32>,
            audit_url: Option<String>,
            project_name: Option<String>,
            project_website: Option<String>,
            github: Option<String>,
        ) -> Result<SmartContract> {
            let mut smart_contract: SmartContract = self.show(id)?;
            let caller: AccountId = Self::env().caller();
            Self::authorise(smart_contract.caller, caller)?;
            self.validate_ownership_of_azero_id(azero_id.clone(), caller)?;
            if let Some(group_id_unwrapped) = group_id {
                self.validate_membership(group_id_unwrapped, caller)?;
            };

            smart_contract.enabled = enabled;
            smart_contract.azero_id = azero_id.clone();
            smart_contract.group_id = group_id;
            smart_contract.audit_url = audit_url.clone();
            smart_contract.project_name = project_name.clone();
            smart_contract.project_website = project_website.clone();
            smart_contract.github = github.clone();
            self.smart_contracts
                .insert(smart_contract.id, &smart_contract);

            // emit event
            Self::emit_event(
                self.env(),
                Event::Update(Update {
                    id: smart_contract.id,
                    enabled: smart_contract.enabled,
                    azero_id,
                    group_id,
                    project_name,
                    project_website,
                    github,
                    audit_url,
                }),
            );

            Ok(smart_contract)
        }

        #[ink(message)]
        pub fn update_fee(&mut self, fee: Balance) -> Result<Balance> {
            Self::authorise(self.admin, Self::env().caller())?;

            self.fee = fee;

            Ok(self.fee)
        }

        fn authorise(allowed: AccountId, received: AccountId) -> Result<()> {
            if allowed != received {
                return Err(AZSmartContractHubError::Unauthorised);
            }

            Ok(())
        }

        // 1. For unit-testing always return the caller.
        // 2. For e2e-testing, I can't write integration tests as the azero.id contract is private.
        // Test different situations safely by returning results based on an azero_id_router_address that is impossible in production
        // with azero.ids that are not allowed in production.
        fn address_by_azero_id(&self, domain: String) -> Result<AccountId> {
            match cfg!(test) {
                true => Ok(self.env().caller()),
                false => {
                    if self.azero_id_router_address
                        == AccountId::try_from(*b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap()
                    {
                        if domain == *MOCK_VALID_AZERO_ID {
                            Ok(self.env().caller())
                        } else if domain == *MOCK_INVALID_AZERO_ID {
                            Ok(self.env().account_id())
                        } else {
                            Err(AZSmartContractHubError::NotFound("Domain".to_string()))
                        }
                    } else {
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

        fn emit_event<EE: EmitEvent<Self>>(emitter: EE, event: Event) {
            emitter.emit_event(event);
        }

        fn format_url(&self, url: String) -> String {
            url.trim().to_string()
        }

        // For unit-testing always return Ok.
        fn validate_membership(&self, group_id: u32, account: AccountId) -> Result<Role> {
            match cfg!(test) {
                true => Ok(Role::Member),
                false => {
                    const VALIDATE_MEMBERSHIP_SELECTOR: [u8; 4] =
                        ink::selector_bytes!("validate_membership");
                    Ok(build_call::<Environment>()
                        .call(self.az_groups_address)
                        .exec_input(
                            ExecutionInput::new(Selector::new(VALIDATE_MEMBERSHIP_SELECTOR))
                                .push_arg(group_id)
                                .push_arg(account),
                        )
                        .returns::<core::result::Result<Role, AZGroupsError>>()
                        .invoke()?)
                }
            }
        }

        fn validate_ownership_of_azero_id(
            &self,
            azero_id: String,
            caller: AccountId,
        ) -> Result<()> {
            if caller != self.address_by_azero_id(azero_id.clone())? {
                return Err(AZSmartContractHubError::UnprocessableEntity(
                    "Domain does not belong to caller".to_string(),
                ));
            }

            Ok(())
        }

        fn validate_presence_of(string: &str, field_name: &str) -> Result<()> {
            if string.is_empty() {
                return Err(AZSmartContractHubError::UnprocessableEntity(format!(
                    "{field_name} can't be blank"
                )));
            };

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        const MOCK_AZERO_ID: &str = "OnionKnight";
        const MOCK_AZERO_ID_TWO: &str = "Robert Ford";
        const MOCK_ABI_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/tmuurccd5a7lcvin6ae9.json";
        const MOCK_CONTRACT_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/vsvsvavdvavav.json";
        const MOCK_WASM_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/ffbrgnteyjytntehthw34hhhwhwhwnq343.json";
        const MOCK_AUDIT_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/mlkmkbdsbmdsb3rrg3m.json";
        const MOCK_PROJECT_NAME: &str = "Smart Contract Hub";
        const MOCK_PROJECT_WEBSITE: &str = "https://someprojectwebsite.org/projects/project-name";
        const MOCK_GITHUB: &str = "https://github.com/smart-contract-hub/project-name";

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

        #[ink::test]
        fn test_show() {
            let (accounts, mut az_smart_contract_hub) = init();
            // = when smart_contract does not exist
            // * it returns error
            assert_eq!(
                az_smart_contract_hub.show(0),
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string()
                ))
            );
            // = when smart_contract exists
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(
                az_smart_contract_hub.fee,
            );
            let smart_contract: SmartContract = az_smart_contract_hub
                .create(
                    accounts.alice,
                    0,
                    MOCK_AZERO_ID.to_string(),
                    MOCK_ABI_URL.to_string(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            // = * it returns the smart_contract
            assert_eq!(
                az_smart_contract_hub.show(smart_contract.id),
                Ok(smart_contract)
            );
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_create() {
            let (accounts, mut az_smart_contract_hub) = init();
            // when smart_contracts_count is u32::MAX
            az_smart_contract_hub.smart_contracts_count = u32::MAX;
            // * it raises an error
            let result = az_smart_contract_hub.create(
                accounts.alice,
                0,
                MOCK_AZERO_ID.to_string(),
                MOCK_ABI_URL.to_string(),
                Some(MOCK_CONTRACT_URL.to_string()),
                Some(MOCK_WASM_URL.to_string()),
                Some(MOCK_AUDIT_URL.to_string()),
                Some(5),
                Some(MOCK_PROJECT_NAME.to_string()),
                Some(MOCK_PROJECT_WEBSITE.to_string()),
                Some(MOCK_GITHUB.to_string()),
            );
            assert_eq!(
                result,
                Err(AZSmartContractHubError::UnprocessableEntity(
                    "Smart contract limit reached".to_string(),
                ))
            );
            // when smart_contracts_count is less than u32::MAX
            // * tested below
        }

        #[ink::test]
        fn test_update() {
            let (accounts, mut az_smart_contract_hub) = init();
            // = when smart_contract doesn't exist
            // = * it raises an error
            let mut result = az_smart_contract_hub.update(
                0,
                false,
                MOCK_AZERO_ID.to_string(),
                None,
                None,
                None,
                None,
                None,
            );
            assert_eq!(
                result,
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string()
                ))
            );

            // = when smart_contract exists
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(
                az_smart_contract_hub.fee,
            );
            az_smart_contract_hub
                .create(
                    accounts.alice,
                    0,
                    MOCK_AZERO_ID.to_string(),
                    MOCK_ABI_URL.to_string(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            // == when called by account that is not the original caller
            set_caller::<DefaultEnvironment>(accounts.charlie);
            // == * it raises an error
            result = az_smart_contract_hub.update(
                0,
                false,
                MOCK_AZERO_ID.to_string(),
                None,
                None,
                None,
                None,
                None,
            );
            assert_eq!(result, Err(AZSmartContractHubError::Unauthorised));
            // == when called by account that is the original caller
            set_caller::<DefaultEnvironment>(accounts.bob);
            result = az_smart_contract_hub.update(
                0,
                false,
                MOCK_AZERO_ID_TWO.to_string(),
                Some(412),
                Some(MOCK_AUDIT_URL.to_string()),
                Some(MOCK_PROJECT_NAME.to_string()),
                Some(MOCK_PROJECT_WEBSITE.to_string()),
                Some(MOCK_GITHUB.to_string()),
            );
            let result_unwrapped = result.unwrap();
            // == * it updates the enabled status
            assert_eq!(result_unwrapped.enabled, false);
            // == * it updates the azero id
            assert_eq!(result_unwrapped.azero_id, MOCK_AZERO_ID_TWO.to_string());
            // == * it updates the group id
            assert_eq!(result_unwrapped.group_id, Some(412));
            // == * it updates the audit url
            assert_eq!(result_unwrapped.audit_url, Some(MOCK_AUDIT_URL.to_string()));
            // == * it updates the project name
            assert_eq!(
                result_unwrapped.project_name,
                Some(MOCK_PROJECT_NAME.to_string())
            );
            // == * it updates the project website
            assert_eq!(
                result_unwrapped.project_website,
                Some(MOCK_PROJECT_WEBSITE.to_string())
            );
            // == * it updates the github
            assert_eq!(result_unwrapped.github, Some(MOCK_GITHUB.to_string()));
        }

        #[ink::test]
        fn test_update_fee() {
            let (accounts, mut az_smart_contract_hub) = init();
            // when called by non admin
            set_caller::<DefaultEnvironment>(accounts.django);
            // * it raises an error
            let result = az_smart_contract_hub.update_fee(5);
            assert_eq!(result, Err(AZSmartContractHubError::Unauthorised));
            // when called by the admin
            set_caller::<DefaultEnvironment>(accounts.bob);
            // * it updates the fee
            az_smart_contract_hub.update_fee(5).unwrap();
            assert_eq!(az_smart_contract_hub.fee, 5)
        }
    }

    // The main purpose of the e2e tests are to test the interactions with az groups contract
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use crate::az_smart_contract_hub::AZSmartContractHubRef;
        use az_groups::AZGroupsRef;
        use ink_e2e::build_message;
        use ink_e2e::Keypair;

        // === CONSTANTS ===
        const MOCK_ABSENT_AZERO_ID: &str = "MOCK ABSENT AZERO ID";
        const MOCK_ABI_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/tmuurccd5a7lcvin6ae9.json";
        const MOCK_CONTRACT_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/vsvsvavdvavav.json";
        const MOCK_WASM_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/ffbrgnteyjytntehthw34hhhwhwhwnq343.json";
        const MOCK_AUDIT_URL: &str = "https://res.mockcdn.com/xasdf123/raw/upload/v1690808298/smart_contract_hub/mlkmkbdsbmdsb3rrg3m.json";
        const MOCK_PROJECT_NAME: &str = "Smart Contract Hub";
        const MOCK_PROJECT_WEBSITE: &str = "https://someprojectwebsite.org/projects/project-name";
        const MOCK_GITHUB: &str = "https://github.com/smart-contract-hub/project-name";

        // === TYPES ===
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        // === HELPERS ===
        fn account_id(k: Keypair) -> AccountId {
            AccountId::try_from(k.public_key().to_account_id().as_ref())
                .expect("account keyring has a valid account id")
        }

        fn mock_azero_id_router_address() -> AccountId {
            AccountId::try_from(*b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap()
        }

        // === HANDLES ===
        #[ink_e2e::test]
        async fn test_create(mut client: ::ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate AZ Groups
            let az_groups_contstructor = AZGroupsRef::new();
            let az_groups_account_id = client
                .instantiate(
                    "az_groups",
                    &ink_e2e::alice(),
                    az_groups_contstructor,
                    0,
                    None,
                )
                .await
                .expect("AZ Groups instantiate failed")
                .account_id;

            // Instantiate AZSmartContractHub
            let az_smart_contract_hub_constructor =
                AZSmartContractHubRef::new(mock_azero_id_router_address(), az_groups_account_id);
            let az_smart_contract_hub_id = client
                .instantiate(
                    "az_smart_contract_hub",
                    &ink_e2e::eve(),
                    az_smart_contract_hub_constructor,
                    0,
                    None,
                )
                .await
                .expect("AZ Smart Contract Hub instantiate failed")
                .account_id;

            // when count is u32::MAX
            // * tested above
            // when count is less than u32::MAX
            // = when azero id does not exist
            let mut create_message = build_message::<AZSmartContractHubRef>(
                az_smart_contract_hub_id.clone(),
            )
            .call(|az_smart_contract_hub| {
                az_smart_contract_hub.create(
                    account_id(ink_e2e::eve()),
                    0,
                    MOCK_ABSENT_AZERO_ID.to_string(),
                    MOCK_ABI_URL.to_string(),
                    Some(MOCK_CONTRACT_URL.to_string()),
                    Some(MOCK_WASM_URL.to_string()),
                    Some(MOCK_AUDIT_URL.to_string()),
                    Some(5),
                    Some(MOCK_PROJECT_NAME.to_string()),
                    Some(MOCK_PROJECT_WEBSITE.to_string()),
                    Some(MOCK_GITHUB.to_string()),
                )
            });
            let mut result = client
                .call_dry_run(&ink_e2e::alice(), &create_message, 0, None)
                .await
                .return_value();
            // = * it raises an error
            assert_eq!(
                result,
                Err(AZSmartContractHubError::NotFound("Domain".to_string()))
            );
            // = when azero id exists
            // == when caller doesn't own azero id
            // == * it raises an error
            create_message = build_message::<AZSmartContractHubRef>(
                az_smart_contract_hub_id.clone(),
            )
            .call(|az_smart_contract_hub| {
                az_smart_contract_hub.create(
                    account_id(ink_e2e::eve()),
                    0,
                    MOCK_INVALID_AZERO_ID.to_string(),
                    MOCK_ABI_URL.to_string(),
                    Some(MOCK_CONTRACT_URL.to_string()),
                    Some(MOCK_WASM_URL.to_string()),
                    Some(MOCK_AUDIT_URL.to_string()),
                    Some(5),
                    Some(MOCK_PROJECT_NAME.to_string()),
                    Some(MOCK_PROJECT_WEBSITE.to_string()),
                    Some(MOCK_GITHUB.to_string()),
                )
            });
            result = client
                .call_dry_run(&ink_e2e::alice(), &create_message, 0, None)
                .await
                .return_value();
            assert_eq!(
                result,
                Err(AZSmartContractHubError::UnprocessableEntity(
                    "Domain does not belong to caller".to_string()
                ))
            );

            // == when caller owns azero id
            // === when group does not exist
            // === * it raises an error
            create_message = build_message::<AZSmartContractHubRef>(
                az_smart_contract_hub_id.clone(),
            )
            .call(|az_smart_contract_hub| {
                az_smart_contract_hub.create(
                    account_id(ink_e2e::eve()),
                    0,
                    MOCK_VALID_AZERO_ID.to_string(),
                    MOCK_ABI_URL.to_string(),
                    Some(MOCK_CONTRACT_URL.to_string()),
                    Some(MOCK_WASM_URL.to_string()),
                    Some(MOCK_AUDIT_URL.to_string()),
                    Some(0),
                    Some(MOCK_PROJECT_NAME.to_string()),
                    Some(MOCK_PROJECT_WEBSITE.to_string()),
                    Some(MOCK_GITHUB.to_string()),
                )
            });
            result = client
                .call_dry_run(&ink_e2e::alice(), &create_message, 0, None)
                .await
                .return_value();
            assert_eq!(
                result,
                Err(AZSmartContractHubError::AZGroupsError(
                    AZGroupsError::NotFound("Group".to_string())
                ))
            );
            // === when group exists
            let mut create_group_message =
                build_message::<AZGroupsRef>(az_groups_account_id.clone())
                    .call(|az_groups| az_groups.groups_create("Eve's team".to_string()));
            let mut groups_result = client
                .call(&ink_e2e::eve(), create_group_message, 0, None)
                .await
                .unwrap()
                .dry_run
                .exec_result
                .result;
            assert!(groups_result.is_ok());
            // ==== when user isn't a member of the group
            result = client
                .call_dry_run(&ink_e2e::alice(), &create_message, 0, None)
                .await
                .return_value();
            assert_eq!(
                result,
                Err(AZSmartContractHubError::AZGroupsError(
                    AZGroupsError::NotFound("GroupUser".to_string())
                ))
            );
            // ==== when user is a member of the group
            create_group_message = build_message::<AZGroupsRef>(az_groups_account_id.clone())
                .call(|az_groups| az_groups.groups_create("Alice's team".to_string()));
            groups_result = client
                .call(&ink_e2e::alice(), create_group_message, 0, None)
                .await
                .unwrap()
                .dry_run
                .exec_result
                .result;
            assert!(groups_result.is_ok());
            // ===== when abi url is empty or made of only white space
            // ===== * it raises an error
            create_message = build_message::<AZSmartContractHubRef>(
                az_smart_contract_hub_id.clone(),
            )
            .call(|az_smart_contract_hub| {
                az_smart_contract_hub.create(
                    account_id(ink_e2e::eve()),
                    0,
                    MOCK_VALID_AZERO_ID.to_string(),
                    " ".to_string(),
                    Some(MOCK_CONTRACT_URL.to_string()),
                    Some(MOCK_WASM_URL.to_string()),
                    Some(MOCK_AUDIT_URL.to_string()),
                    Some(1),
                    Some(MOCK_PROJECT_NAME.to_string()),
                    Some(MOCK_PROJECT_WEBSITE.to_string()),
                    Some(MOCK_GITHUB.to_string()),
                )
            });
            result = client
                .call_dry_run(&ink_e2e::alice(), &create_message, 0, None)
                .await
                .return_value();
            assert_eq!(
                result,
                Err(AZSmartContractHubError::UnprocessableEntity(format!(
                    "Link to abi can't be blank"
                )))
            );
            // ===== when abi url is present
            let mut mock_abi_url_with_whitespaces: String = " ".to_string();
            mock_abi_url_with_whitespaces.push_str(&MOCK_ABI_URL.to_string());
            mock_abi_url_with_whitespaces.push_str(&" ".to_string());
            create_message = build_message::<AZSmartContractHubRef>(
                az_smart_contract_hub_id.clone(),
            )
            .call(|az_smart_contract_hub| {
                az_smart_contract_hub.create(
                    account_id(ink_e2e::eve()),
                    0,
                    MOCK_VALID_AZERO_ID.to_string(),
                    mock_abi_url_with_whitespaces.clone(),
                    Some(MOCK_CONTRACT_URL.to_string()),
                    Some(MOCK_WASM_URL.to_string()),
                    Some(MOCK_AUDIT_URL.to_string()),
                    Some(1),
                    Some(MOCK_PROJECT_NAME.to_string()),
                    Some(MOCK_PROJECT_WEBSITE.to_string()),
                    Some(MOCK_GITHUB.to_string()),
                )
            });
            // ====== when transferred value does not equal fee
            result = client
                .call_dry_run(&ink_e2e::alice(), &create_message, 0, None)
                .await
                .return_value();
            assert_eq!(
                result,
                Err(AZSmartContractHubError::UnprocessableEntity(format!(
                    "Incorrect fee"
                )))
            );

            // ====== when transferred value equals fee
            let eve_balance: Balance = client.balance(account_id(ink_e2e::eve())).await.unwrap();
            let result = client
                .call(&ink_e2e::alice(), create_message, 1_000, None)
                .await
                .expect("Create failed");
            let result_unwrapped: SmartContract = result.dry_run.return_value().unwrap();
            // ====== * it sends the balance to the admin
            let new_eve_balance: Balance =
                client.balance(account_id(ink_e2e::eve())).await.unwrap();
            assert_eq!(new_eve_balance, eve_balance + 1_000);

            // ====== * it stores the id as the current length
            assert_eq!(result_unwrapped.id, 0);
            // ====== * it increases the smart_contracts length by 1
            let config_message =
                build_message::<AZSmartContractHubRef>(az_smart_contract_hub_id.clone())
                    .call(|az_smart_contract_hub| az_smart_contract_hub.config());
            let config_result = client
                .call_dry_run(&ink_e2e::alice(), &config_message, 0, None)
                .await
                .return_value();
            assert_eq!(config_result.smart_contracts_count, 1);
            // ====== * it stores the smart_contract
            let show_message =
                build_message::<AZSmartContractHubRef>(az_smart_contract_hub_id.clone())
                    .call(|az_smart_contract_hub| az_smart_contract_hub.show(0));
            let show_result = client
                .call_dry_run(&ink_e2e::alice(), &show_message, 0, None)
                .await
                .return_value();
            assert_eq!(show_result.unwrap(), result_unwrapped);
            // ====== * it stores the submitted smart contract address
            assert_eq!(
                result_unwrapped.smart_contract_address,
                account_id(ink_e2e::eve())
            );
            // ====== * it sets the chain
            assert_eq!(result_unwrapped.chain, 0);
            // ====== * it sets the azero id domain
            assert_eq!(result_unwrapped.azero_id, MOCK_VALID_AZERO_ID.to_string());
            // ====== * it sets the abi url with trimmed whitespaces
            assert_eq!(result_unwrapped.abi_url, MOCK_ABI_URL.to_string());
            // ====== * it sets the contract url
            assert_eq!(
                result_unwrapped.contract_url,
                Some(MOCK_CONTRACT_URL.to_string())
            );
            // ====== * it sets the wasm url
            assert_eq!(result_unwrapped.wasm_url, Some(MOCK_WASM_URL.to_string()));
            // ====== * it sets the audit url
            assert_eq!(result_unwrapped.audit_url, Some(MOCK_AUDIT_URL.to_string()));
            // ====== * it sets the group_id
            assert_eq!(result_unwrapped.group_id, Some(1));
            // ====== * it sets the project name
            assert_eq!(
                result_unwrapped.project_name,
                Some(MOCK_PROJECT_NAME.to_string())
            );
            // ====== * it sets the project name
            assert_eq!(
                result_unwrapped.project_website,
                Some(MOCK_PROJECT_WEBSITE.to_string())
            );
            // ====== * it sets the github
            assert_eq!(result_unwrapped.github, Some(MOCK_GITHUB.to_string()));
            // ====== * it sets the caller to the caller
            assert_eq!(result_unwrapped.caller, account_id(ink_e2e::alice()));

            Ok(())
        }
    }
}
