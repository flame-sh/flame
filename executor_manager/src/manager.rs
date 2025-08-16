/*
Copyright 2025 Flame Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
 */

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::client::BackendClient;
use crate::executor::{self, Executor, ExecutorPtr};
use common::apis::Node;
use common::lock_ptr;
use common::{ctx::FlameContext, FlameError};

pub struct ExecutorManager {
    ctx: FlameContext,
    executors: HashMap<String, ExecutorPtr>,
    client: BackendClient,
}

impl ExecutorManager {
    pub async fn new(ctx: &FlameContext) -> Result<Self, FlameError> {
        // Create the Flame directory.
        fs::create_dir_all("/tmp/flame/shim")
            .map_err(|e| FlameError::Internal(format!("failed to create shim directory: {e}")))?;

        let client = BackendClient::new(ctx).await?;

        Ok(Self {
            ctx: ctx.clone(),
            executors: HashMap::new(),
            client,
        })
    }

    pub async fn run(&mut self) -> Result<(), FlameError> {
        let mut node = Node::new();
        self.client.register_node(&node).await?;

        loop {
            let mut allocations = self.client.watch_allocation(&node.name).await?;

            {
                for executor in self.executors.values() {
                    let executor = lock_ptr!(executor)?;
                    for allocation in &allocations {
                        if executor.resreq == allocation.resource_requirement {
                            if allocation.replica > 0 {
                                allocation.replica -= 1;
                            } else {
                                // Remove the executor from the map.
                                // The executor will be stopped by the executor itself.
                                self.executors.remove(&executor.id);
                            }
                        }
                    }
                }
            }

            for allocation in allocations {
                let replica = allocation.replica;
                let resreq = allocation.resource_requirement;
                log::debug!(
                    "Starting {} executors for resource requirement: {:?}",
                    replica,
                    resreq
                );
                for _ in 0..replica {
                    let executor = Executor::new(self.client.clone(), resreq.clone());
                    let executor_ptr = Arc::new(Mutex::new(executor));
                    self.executors
                        .insert(executor.id.clone(), executor_ptr.clone());
                    log::debug!("Starting executor: {}", executor.id);
                    executor::start(executor_ptr.clone());
                }
            }

            node.refresh();
            self.client.update_node(&node).await?;
        }

        Ok(())
    }
}
