// Copyright 2016 Nexus Development
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Quick contract runner for testing.

extern crate docopt;
extern crate serde_json;
extern crate rustc_serialize;

extern crate ethcore;
extern crate ethcore_util as util;
extern crate ethcore_devtools as devtools;
extern crate ethcore_io as io;
extern crate ethcore_logger as logger;
extern crate ethstore;

use devtools::RandomTempPath;

use ethcore::transaction::{Transaction, Action};
use ethcore::account_provider::AccountProvider;
use ethcore::client::{Client, ClientConfig, MiningBlockChainClient};
use ethcore::spec::Spec;
use ethcore::miner::Miner;

use ethstore::ethkey::Secret;

use io::IoChannel;

use util::{U256, FromHex, Uint};

use std::str::FromStr;
use std::sync::Arc;

const USAGE: &'static str = r#"
Quick contract runner
Usage:
  parity-quickrun <data> [options]

Options:
  --gas GAS           Supplied gas
  -h --help           Show this screen
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
  arg_data: String,
  flag_gas: Option<u64>,
}

impl Args {
  pub fn gas(&self) -> U256 {
    self.flag_gas
      .clone()
      .and_then(|g| Some(U256::from(g)))
      .unwrap_or_else(|| !U256::zero())
  }
}

fn main() {
  let log_config = logger::Config {
    mode: None,
    color: true,
    file: None
  };
  
  logger::setup_log(&log_config).unwrap();

  let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode())
    .unwrap_or_else(|e| e.exit());
  println!("{:?}", args);

  let x: serde_json::Value = serde_json::from_reader(
    std::io::stdin()
  ).unwrap();

  let root = x.as_object().unwrap();
  let bin_hex = root.get("bin").unwrap().as_string().unwrap();
  let bin = bin_hex.to_string().from_hex().ok().unwrap();
  // let abi_json = root.get("abi").unwrap().as_string().unwrap();
  // let abi: serde_json::Value = serde_json::from_str(abi_json).unwrap();

  let spec = Spec::load(include_bytes!("./chain.json"));

  let temp = RandomTempPath::new();
  let path = temp.as_path();

  let miner = Arc::new(Miner::with_spec(&spec));
  let client = Client::new(
    ClientConfig::default(),
    &spec,
    &path,
    miner,
    IoChannel::disconnected()
  ).unwrap();

  let secret = Secret::from_str(
    "a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65"
  ).unwrap();
  
  // let kp = KeyPair::from_secret(secret).unwrap();

  let account_provider = AccountProvider::transient_provider();
  let account = account_provider.insert_account(secret.clone(), "").unwrap();
  account_provider.unlock_account_permanently(account, "".to_string()).unwrap();

  println!("{:?}", account);

  let transaction = Transaction {
    action: Action::Create,
    value: U256::from(0),
    data: bin,
    gas: args.gas(),
    gas_price: U256::one(),
    nonce: U256::from(1)
  };

  let signed_transaction = transaction.sign(&secret);

  let mut open_block = client.prepare_open_block(
    account,
    (1.into(), 1_000_000.into()),
    vec![]
  );

  {
    let receipt = open_block.push_transaction(signed_transaction, None).unwrap();
    println!("{:?}", receipt);
  }

  open_block.close();
  client.flush_queue();
}
