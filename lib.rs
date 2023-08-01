#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_smart_contract_metadata_hub {
    use ink::storage::Mapping;

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AzSmartContractMetadataHubError {
        RecordsLimitReached,
    }

    // === EVENTS (To be used with Subsquid) ===

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Record {
        id: u32,
        smart_contract_address: AccountId,
        likes: u16,
        dislikes: u16,
    }

    #[derive(Debug, Default)]
    #[ink::storage_item]
    pub struct Records {
        values: Mapping<u32, Record>,
        length: u32,
    }
    impl Records {
        //     pub fn index(&self, page: u32, size: u16) -> Vec<Record> {
        //         let mut records: Vec<Record> = vec![];
        //         // When there's no records
        //         if self.length == 0 {
        //             return records;
        //         }

        //         let records_to_skip: Option<u32> = if page == 0 {
        //             Some(0)
        //         } else {
        //             page.checked_mul(size.into())
        //         };
        //         let starting_index: u32;
        //         let ending_index: u32;
        //         // When the records to skip is greater than max possible
        //         if let Some(records_to_skip_unwrapped) = records_to_skip {
        //             let ending_index_wrapped: Option<u32> =
        //                 self.length.checked_sub(records_to_skip_unwrapped);
        //             // When records to skip is greater than total number of records
        //             if ending_index_wrapped.is_none() {
        //                 return records;
        //             }
        //             ending_index = ending_index_wrapped.unwrap();
        //             starting_index = ending_index.saturating_sub(size.into());
        //         } else {
        //             return records;
        //         }
        //         for i in (starting_index..=ending_index).rev() {
        //             records.push(self.values.get(i).unwrap())
        //         }
        //         records
        //     }

        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            if self.length == u32::MAX {
                return Err(AzSmartContractMetadataHubError::RecordsLimitReached);
            }

            let record: Record = Record {
                id: self.length,
                smart_contract_address,
                likes: 1,
                dislikes: 0,
            };
            self.values.insert(self.length, &record);
            self.length += 1;

            Ok(record)
        }

        //     pub fn update(&mut self, value: &Record) {
        //         self.values.insert(value.id, value);
        //     }
    }

    // === CONTRACT ===
    #[ink(storage)]
    #[derive(Default)]
    pub struct AzSmartContractMetadataHub {
        records: Records,
    }
    impl AzSmartContractMetadataHub {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                records: Records {
                    values: Mapping::default(),
                    length: 0,
                },
            }
        }

        #[ink(message)]
        pub fn show(&self, id: u32) -> Option<Record> {
            self.records.values.get(id)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

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
        #[ink::test]
        fn test_new() {
            let (_accounts, az_smart_contract_metadata_hub) = init();
            // * it sets records
            assert_eq!(az_smart_contract_metadata_hub.records.length, 0);
        }

        #[ink::test]
        fn test_show() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            // = when record does not exist
            // * it return None
            assert_eq!(az_smart_contract_metadata_hub.show(0), None);
            // = when record exists
            let record: Record = az_smart_contract_metadata_hub
                .records
                .create(accounts.alice)
                .unwrap();
            // * it returns the record
            assert_eq!(az_smart_contract_metadata_hub.show(record.id), Some(record));
        }
    }
}
