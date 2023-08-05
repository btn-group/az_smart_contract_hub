#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_smart_contract_metadata_hub {
    use ink::prelude::string::{String, ToString};
    use ink::storage::Mapping;

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AzSmartContractMetadataHubError {
        NotFound(String),
        OutOfRange(String),
        Unauthorized,
        Unchanged(String),
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct Create {
        id: u32,
        smart_contract_address: AccountId,
        url: String,
        submitter: AccountId,
    }

    #[ink(event)]
    pub struct Toggle {
        id: u32,
        enabled: bool,
    }

    #[ink(event)]
    pub struct Rate {
        id: u32,
        previous_user_rating: i8,
        new_user_rating: i8,
        user: AccountId,
    }

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Record {
        id: u32,
        smart_contract_address: AccountId,
        url: String,
        likes: u16,
        dislikes: u16,
        submitter: AccountId,
        enabled: bool,
    }

    #[derive(Debug, Default)]
    #[ink::storage_item]
    pub struct Records {
        values: Mapping<u32, Record>,
        length: u32,
    }
    impl Records {
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            submitter: AccountId,
            url: String,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            let record: Record = Record {
                id: self.length,
                smart_contract_address,
                url,
                likes: 1,
                dislikes: 0,
                submitter,
                enabled: true,
            };
            self.values.insert(self.length, &record);
            self.length += 1;

            Ok(record)
        }

        pub fn update(&mut self, value: &Record) {
            self.values.insert(value.id, value);
        }
    }

    // === CONTRACT ===
    #[ink(storage)]
    #[derive(Default)]
    pub struct AzSmartContractMetadataHub {
        records: Records,
        user_ratings: Mapping<(u32, AccountId), i8>,
    }
    impl AzSmartContractMetadataHub {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                records: Records {
                    values: Mapping::default(),
                    length: 0,
                },
                user_ratings: Mapping::default(),
            }
        }

        // === QUERIES ===
        #[ink(message)]
        pub fn show(&self, id: u32) -> Result<Record, AzSmartContractMetadataHubError> {
            if let Some(record) = self.records.values.get(id) {
                Ok(record)
            } else {
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string(),
                ))
            }
        }

        // === HANDLES ===
        #[ink(message)]
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            url: String,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            let caller: AccountId = Self::env().caller();
            let record: Record =
                self.records
                    .create(smart_contract_address, caller, url.clone())?;
            self.user_ratings.insert((record.id, caller), &1);

            // emit event
            self.env().emit_event(Create {
                id: record.id,
                smart_contract_address,
                url,
                submitter: caller,
            });

            Ok(record)
        }

        #[ink(message)]
        pub fn rate(
            &mut self,
            id: u32,
            new_user_rating: i8,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            let mut record: Record = self.show(id)?;
            if !(-1..=1).contains(&new_user_rating) {
                return Err(AzSmartContractMetadataHubError::OutOfRange(
                    "Rating".to_string(),
                ));
            }

            let caller: AccountId = Self::env().caller();
            let previous_user_rating: i8 = self.user_ratings.get((id, caller)).unwrap_or(0);
            if previous_user_rating == new_user_rating {
                return Err(AzSmartContractMetadataHubError::Unchanged(
                    "Rating".to_string(),
                ));
            }

            if new_user_rating == -1 {
                if previous_user_rating == 1 {
                    record.likes -= 1
                }
                record.dislikes += 1
            } else if new_user_rating == 0 {
                if previous_user_rating == 1 {
                    record.likes -= 1
                } else if previous_user_rating == -1 {
                    record.dislikes -= 1
                }
            } else {
                if previous_user_rating == -1 {
                    record.dislikes -= 1
                }
                record.likes += 1
            }
            self.records.update(&record);
            self.user_ratings.insert((id, caller), &new_user_rating);

            // emit event
            self.env().emit_event(Rate {
                id: record.id,
                previous_user_rating,
                new_user_rating,
                user: caller,
            });

            Ok(record)
        }

        #[ink(message)]
        pub fn toggle_enabled(
            &mut self,
            id: u32,
            enabled: bool,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            let mut record: Record = self.show(id)?;
            let caller: AccountId = Self::env().caller();
            if caller != record.submitter {
                return Err(AzSmartContractMetadataHubError::Unauthorized);
            }
            if record.enabled == enabled {
                return Err(AzSmartContractMetadataHubError::Unchanged(
                    "Enabled".to_string(),
                ));
            }

            record.enabled = enabled;
            self.records.update(&record);

            // emit event
            self.env().emit_event(Toggle {
                id: record.id,
                enabled: record.enabled,
            });

            Ok(record)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        const MOCK_URL: &str = "https://res.cloudinary.com/hv5cxagki/raw/upload/v1690808298/smart_contract_metadata/tmuurccd5a7lcvin6ae9.json";

        // === HELPERS ===
        fn init() -> (
            DefaultAccounts<DefaultEnvironment>,
            AzSmartContractMetadataHub,
        ) {
            let accounts = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.bob);
            let az_smart_contract_metadata_hub = AzSmartContractMetadataHub::new();
            (accounts, az_smart_contract_metadata_hub)
        }

        // === TESTS ===
        // === TEST QUERIES ===
        #[ink::test]
        fn test_show() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            // = when record does not exist
            // * it returns error
            assert_eq!(
                az_smart_contract_metadata_hub.show(0),
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string()
                ))
            );
            // = when record exists
            let record: Record = az_smart_contract_metadata_hub
                .records
                .create(accounts.alice, accounts.bob, MOCK_URL.to_string())
                .unwrap();
            // = * it returns the record
            assert_eq!(az_smart_contract_metadata_hub.show(record.id), Ok(record));
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_create() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            az_smart_contract_metadata_hub.records.length = u32::MAX - 1;
            // * it stores the submitter as the caller
            let result =
                az_smart_contract_metadata_hub.create(accounts.alice, MOCK_URL.to_string());
            let result_unwrapped = result.unwrap();
            // * it stores the id as the current length
            assert_eq!(result_unwrapped.id, u32::MAX - 1);
            // * it increases the records length by 1
            assert_eq!(az_smart_contract_metadata_hub.records.length, u32::MAX);
            // * it stores the submitted smart contract address
            assert_eq!(result_unwrapped.smart_contract_address, accounts.alice);
            // * it sets the like to 1 and dislike to 0
            assert_eq!(result_unwrapped.likes, 1);
            assert_eq!(result_unwrapped.dislikes, 0);
            // * it sets the submitter to the caller
            assert_eq!(result_unwrapped.submitter, accounts.bob);
            // * it sets the user_rating to 1
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((result_unwrapped.id, result_unwrapped.submitter))
                    .unwrap(),
                1
            );
            // * it stores the record
            assert_eq!(
                result_unwrapped,
                az_smart_contract_metadata_hub
                    .records
                    .values
                    .get(result_unwrapped.id)
                    .unwrap()
            );
        }

        #[ink::test]
        fn test_rate() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            // = when record doesn't exist
            // = * it raises an error
            let mut result = az_smart_contract_metadata_hub.rate(0, 0);
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string()
                ))
            );

            // = when record exists
            az_smart_contract_metadata_hub
                .create(accounts.alice, MOCK_URL.to_string())
                .unwrap();
            // == when new rating is less than -1
            result = az_smart_contract_metadata_hub.rate(0, -2);
            // == * it raises an error
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::OutOfRange(
                    "Rating".to_string()
                ))
            );
            // == when new rating is grater than 1
            result = az_smart_contract_metadata_hub.rate(0, 2);
            // == * it raises an error
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::OutOfRange(
                    "Rating".to_string()
                ))
            );
            // == when new rating is within range
            // === when new rating is the same as user's current rating
            result = az_smart_contract_metadata_hub.rate(0, 1);
            // === * it raises an error
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::Unchanged(
                    "Rating".to_string()
                ))
            );

            // === when new rating is different to user's current rating
            // ==== when current rating is 1
            // ===== when new rating is 0
            let mut new_user_rating: i8 = 0;
            result = az_smart_contract_metadata_hub.rate(0, new_user_rating);
            let mut record = result.unwrap();
            // ===== * it reduces the record's likes by 1
            // ===== * it does not change the dislikes
            // ===== * it sets the user's rating for the record to 1
            assert_eq!(record.likes, 0);
            assert_eq!(record.dislikes, 0);
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ===== when new rating is -1
            az_smart_contract_metadata_hub.rate(0, 1).unwrap();
            new_user_rating = -1;
            record = az_smart_contract_metadata_hub
                .rate(0, new_user_rating)
                .unwrap();
            // ===== * it reduces the record's likes by 1
            // ===== * it increases the record's dislikes by 1
            // ===== * it sets the user's rating for the record to -1
            assert_eq!(record.likes, 0);
            assert_eq!(record.dislikes, 1);
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ==== when current rating is -1
            // ===== when new rating is 0
            new_user_rating = 0;
            result = az_smart_contract_metadata_hub.rate(0, new_user_rating);
            record = result.unwrap();
            // ===== * it reduces the record's dislikes by 1
            // ===== * it does not change the likes
            // ===== * it sets the user's rating for the record to 0
            assert_eq!(record.likes, 0);
            assert_eq!(record.dislikes, 0);
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ===== when new rating is 1
            az_smart_contract_metadata_hub.rate(0, -1).unwrap();
            new_user_rating = 1;
            result = az_smart_contract_metadata_hub.rate(0, new_user_rating);
            record = result.unwrap();
            // ===== * it reduces the record's dislikes by 1
            // ===== * it increases the likes by 1
            // ===== * it sets the user's rating for the record to 1
            assert_eq!(record.likes, 1);
            assert_eq!(record.dislikes, 0);
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ==== when current rating is 0
            az_smart_contract_metadata_hub.rate(0, 0).unwrap();
            // ===== when new rating is 1
            new_user_rating = 1;
            result = az_smart_contract_metadata_hub.rate(0, new_user_rating);
            record = result.unwrap();
            // ===== * it does not change the dislikes
            // ===== * it increases the likes by 1
            // ===== * it sets the user's rating for the record to 1
            assert_eq!(record.likes, 1);
            assert_eq!(record.dislikes, 0);
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            az_smart_contract_metadata_hub.rate(0, 0).unwrap();
            // ===== when new rating is -1
            new_user_rating = -1;
            result = az_smart_contract_metadata_hub.rate(0, new_user_rating);
            record = result.unwrap();
            // ===== * it increases the dislikes by 1
            // ===== * it does not change the likes
            // ===== * it sets the user's rating for the record to -1
            assert_eq!(record.likes, 0);
            assert_eq!(record.dislikes, 1);
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );
        }

        #[ink::test]
        fn test_toggle_enabled() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            // = when record doesn't exist
            // = * it raises an error
            let mut result = az_smart_contract_metadata_hub.toggle_enabled(0, false);
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string()
                ))
            );

            // = when record exists
            az_smart_contract_metadata_hub
                .create(accounts.alice, MOCK_URL.to_string())
                .unwrap();
            // == when called by non-submitter
            set_caller::<DefaultEnvironment>(accounts.charlie);
            // == * it raises an error
            result = az_smart_contract_metadata_hub.toggle_enabled(0, false);
            assert_eq!(result, Err(AzSmartContractMetadataHubError::Unauthorized));
            // == when called by submitter
            set_caller::<DefaultEnvironment>(accounts.bob);
            // === when record is already enabled
            // ==== when the user tries to enable
            // ==== * it raises an error
            result = az_smart_contract_metadata_hub.toggle_enabled(0, true);
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::Unchanged(
                    "Enabled".to_string()
                ))
            );
            // ==== when the user tries to disable
            // ==== * it updates the record enabled to false
            result = az_smart_contract_metadata_hub.toggle_enabled(0, false);
            assert_eq!(result.unwrap().enabled, false);

            // === when record is already disabled
            // ==== when the user tries to disable
            // ==== * it raises an error
            result = az_smart_contract_metadata_hub.toggle_enabled(0, false);
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::Unchanged(
                    "Enabled".to_string()
                ))
            );
            // ==== when the user tries to enable
            // ==== * it updates the record enabled to true
            result = az_smart_contract_metadata_hub.toggle_enabled(0, true);
            assert_eq!(result.unwrap().enabled, true);
        }
    }
}
