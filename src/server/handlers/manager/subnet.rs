// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The shared subnet manager module for all subnet management related RPC method calls.

use crate::config::{ReloadableConfig, Subnet};
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::manager::LotusSubnetManager;
use ipc_sdk::subnet_id::SubnetID;
use std::sync::Arc;

/// The subnet manager connection that holds the subnet config and the manager instance.
pub struct Connection<T: JsonRpcClient> {
    subnet: Subnet,
    manager: LotusSubnetManager<T>,
}

impl<T: JsonRpcClient> Connection<T> {
    pub fn subnet(&self) -> &Subnet {
        &self.subnet
    }

    pub fn manager(&self) -> &LotusSubnetManager<T> {
        &self.manager
    }
}

/// The json rpc subnet manager connection pool. This struct can be shared by all the subnet methods.
/// As such, there is no need to re-init the same SubnetManager for different methods to reuse connections.
pub struct SubnetManagerPool {
    config: Arc<ReloadableConfig>,
}

impl SubnetManagerPool {
    pub fn from_reload_config(reload_config: Arc<ReloadableConfig>) -> Self {
        Self {
            config: reload_config,
        }
    }

    /// Get the connection instance for the subnet.
    pub fn get(&self, subnet: &SubnetID) -> Option<Connection<JsonRpcClientImpl>> {
        let config = self.config.get_config();
        let subnets = &config.subnets;

        match subnets.get(subnet) {
            Some(subnet) => {
                let manager = LotusSubnetManager::from_subnet(subnet);
                Some(Connection {
                    manager,
                    subnet: subnet.clone(),
                })
            }
            None => None,
        }
    }
}
