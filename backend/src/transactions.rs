
// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![ allow( bare_trait_objects ) ]

extern crate serde_json;
extern crate serde;

use serde::{Deserialize, Serialize, Deserializer, Serializer};

use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::{CryptoHash, PublicKey, Hash};
use exonum::messages::Message;
use exonum::storage::Fork;
use exonum_time::schema::TimeSchema;

use POST_SERVICE_ID;
use schema::{CurrencySchema, TimestampEntry};

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    #[fail(display = "Time is up")]
    Timeisup = 4,

    #[fail(display = "Pubkey doesn`t belong to inspector")]
    NotInspector = 5,

    #[fail(display = "Pubkey doesn`t belong to issuer")]
    NotIssuer = 6,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    pub WalletTransactions {
        const SERVICE_ID = POST_SERVICE_ID;

        /// Transfer `amount` of the currency from one wallet to another.
        struct Transfer {
            from:    &PublicKey,
            to:      &PublicKey,
            portfolio_id:  u64,
            seed:    u64,
        }

        /// Issue `amount` of the currency to the `wallet`.
        struct CurrencyIssue {
            pub_key:  &PublicKey,
            currency_id: u64,
            amount:  u64,
            seed:    u64,
        }

        /// Create portfolio
        struct CreatePortfolio {
            portfolio_id:    u64,
            pub_key: &PublicKey,
            currencyId_amount_pair : Vec<Vec<u64> >,
        }
    }
}

impl Transaction for Transfer {
    fn verify(&self) -> bool {
        self.verify_signature(self.from())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.from();
        let receiver = self.to();
        if let Some(portfolio) = schema.portfolio(pub_key) {
            let id = self.portfolio_id();
            ///schema.increase_wallet_balance(wallet, amount, &self.hash(), 0);

            let entry = TimestampEntry::new(&self.hash(), time.unwrap());
            schema.add_timestamp(entry);

            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }
    }
}

impl Transaction for CurrencyIssue {
    fn verify(&self) -> bool {
        //(self.from() != self.to()) && self.verify_signature(self.from())
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        
        let mut schema = CurrencySchema::new(fork);
        let currency_id = self.currency_id();
        let amount = self.amount();
        /// There is no checking for double appending currency
        schema.add_currency(currency_id, amount);
        let entry = TimestampEntry::new(&self.hash(), time.unwrap());
        schema.add_timestamp(entry);
        Ok(())
    }
}

impl Transaction for CreatePortfolio {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();
        let id = self.portfolio_id();
        let currencies = self.currencyId_amount_pair();
        if schema.portfolio(pub_key).is_none(){
            schema.create_portfolio(id, pub_key, currencies);            
            let entry = TimestampEntry::new(&self.hash(), time.unwrap());
            schema.add_timestamp(entry);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        } 
    }    
}
