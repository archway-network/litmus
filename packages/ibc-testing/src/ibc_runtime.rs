use crate::chain::*;
use archway_rpc::utils::Coin;
use archway_rpc::{CosmosRPC, Id};
use bollard::container::{CreateContainerOptions, RemoveContainerOptions, StartContainerOptions};
use bollard::models::{HostConfig, PortBinding};
use bollard::Docker;
use std::collections::HashMap;
use std::time::Duration;

pub const ADDR_PREFIX: &str = "archway";
pub const TOKEN: &str = "stake";
pub(crate) const FAUCET_MNEMONIC: &str = "any giant turtle pioneer frequent frown harvest ancient episode junior vocal rent shrimp icon idle echo suspect clean cage eternal sample post heavy enough";

pub fn coin(amount: u128) -> Coin {
    Coin::new(TOKEN.to_string(), amount)
}

#[derive(Default)]
/// Runtime builder, allows for rpc port routing
pub struct IbcRuntimeBuilder {
    pub chain1_rpc_port: Option<usize>,
    pub chain2_rpc_port: Option<usize>,
}

impl IbcRuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chain1_rpc_port(mut self, port: usize) -> Self {
        self.chain1_rpc_port = Some(port);
        self
    }

    pub fn with_chain2_rpc_port(mut self, port: usize) -> Self {
        self.chain2_rpc_port = Some(port);
        self
    }

    /// Create IBC runtime
    pub async fn build<'a>(self, container_id: Option<String>) -> archway_rpc::Result<IbcRuntime> {
        let docker = Docker::connect_with_defaults().unwrap();

        let chain1_port_route = self.chain1_rpc_port.unwrap_or(CHAIN1.rpc_port);
        let chain2_port_route = self.chain2_rpc_port.unwrap_or(CHAIN2.rpc_port);

        let container_id = if let Some(id) = container_id {
            id
        } else {
            // Set up port bindings
            let chain1_port = format!("{}/tcp", CHAIN1.rpc_port);
            let chain2_port = format!("{}/tcp", CHAIN2.rpc_port);
            let mut port_bindings = HashMap::new();
            port_bindings.insert(
                chain1_port.clone(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(chain1_port_route.to_string()),
                }]),
            );
            port_bindings.insert(
                chain2_port.clone(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(chain2_port_route.to_string()),
                }]),
            );

            let config = bollard::container::Config {
                image: Some("ibc-demo:latest"),
                exposed_ports: Some(HashMap::from([
                    (chain1_port.as_str(), HashMap::new()),
                    (chain2_port.as_str(), HashMap::new()),
                ])),
                host_config: Some(HostConfig {
                    port_bindings: Some(port_bindings),
                    ..Default::default()
                }),
                ..Default::default()
            };

            let container_id = docker
                .create_container(None::<CreateContainerOptions<String>>, config)
                .await
                .unwrap()
                .id;

            // TODO: impl a healthcheck
            docker
                .start_container(&container_id, None::<StartContainerOptions<String>>)
                .await
                .unwrap();

            container_id
        };

        let runtime = IbcRuntime {
            chain1: ChainClient::new(
                CHAIN1.id.parse::<Id>().unwrap(),
                format!("http://0.0.0.0:{}", chain1_port_route).as_str(),
            )
            .await?,
            chain2: ChainClient::new(
                CHAIN2.id.parse::<Id>().unwrap(),
                format!("http://0.0.0.0:{}", chain2_port_route).as_str(),
            )
            .await?,
            docker,
            container_id,
        };

        // Wait for both chains to be initialized
        // sleep(Duration::from_secs(10)).await;
        runtime
            .chain1
            .client
            .poll_for_first_block(Duration::from_secs(30))
            .await?;
        runtime
            .chain2
            .client
            .poll_for_first_block(Duration::from_secs(30))
            .await?;

        Ok(runtime)
    }
}

/// Ibc runtime
pub struct IbcRuntime {
    pub chain1: ChainClient,
    pub chain2: ChainClient,
    pub docker: Docker,
    pub container_id: String,
}

impl IbcRuntime {
    /// Stops the docker container, important since you won't be able to run the same ports if you don't
    pub async fn stop(&self) {
        self.docker
            .stop_container(&self.container_id, None)
            .await
            .unwrap();
        self.docker
            .remove_container(&self.container_id, None::<RemoveContainerOptions>)
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::IbcRuntimeBuilder;

    #[tokio::test]
    async fn init() {
        let runtime = IbcRuntimeBuilder::new()
            .with_chain1_rpc_port(27015)
            .with_chain2_rpc_port(27035)
            .build(None)
            .await
            .unwrap();
        dbg!(&runtime.container_id);

        let acc = runtime.chain1.new_account(1000).await.unwrap();
        assert_ne!(acc.account_info.unwrap().number, 0);

        runtime.stop().await;
    }
}
