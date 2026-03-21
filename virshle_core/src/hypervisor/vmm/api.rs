use super::VmmMethods;
use crate::config::init::MANAGED_DIR;
use crate::hypervisor::{
    vmm::{VmConfig, VmInfoResponse, VmRemoveDeviceData, VmState},
    Vm,
};

// Http
use crate::hypervisor::vmm::types::NetConfig;
use hyper::StatusCode;
use virshle_network::{
    connection::{Connection, ConnectionHandle, UnixConnection},
    http::{Rest, RestClient},
};

// Error Handling
use miette::{IntoDiagnostic, Result};
use tracing::{debug, error, info, trace};
use virshle_error::{LibError, VirshleError};

// Convenience methods on top of raw simple cloud-hypervisor API methods.
impl VmmMethods<'_> {
    /// Return vm vsocket path for host guest (ssh) communication.
    /// Return vm socket path for ch REST API communication.
    pub fn get_socket(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/ch.sock", self.vm.uuid);
        Ok(path)
    }
    pub fn get_socket_uri(&self) -> Result<String, VirshleError> {
        let socket = self.get_socket().unwrap();
        let uri = format!("unix://{socket}");
        Ok(uri)
    }
    pub async fn refresh_networks(&mut self) -> Result<(), VirshleError> {
        self._remove_networks().await?;
        Ok(())
    }
    /// Remove network:
    /// - remove config from vmm process.
    /// - remove network device from host.
    async fn _remove_networks(&mut self) -> Result<(), VirshleError> {
        let config = VmConfig::from(self.vm).await?;
        if let Some(networks) = config.net {
            for e in networks {
                if let Some(id) = e.id {
                    self.api()?.remove_device(&id).await?;
                    // Delete network from host.
                }
            }
        }
        Ok(())
    }
}

impl VmmMethods<'_> {
    pub fn api(&self) -> Result<VmmApiMethods<'_>, VirshleError> {
        let conn: Connection = self.vm.try_into()?;
        let mut rest: RestClient = conn.into();
        rest.base_url("/api/v1");
        rest.ping_url(&format!("{}{}", "/api/v1", "/vmm.ping"));
        Ok(VmmApiMethods {
            vm: self.vm,
            client: rest,
        })
    }
}
pub struct VmmApiMethods<'a> {
    vm: &'a Vm,
    client: RestClient,
}
/// See cloud-hypervisor docs/api
/// Some methods doesn't expect any answer in body.
impl VmmApiMethods<'_> {
    /// If we can't establish connection to socket,
    /// this means cloud-hypervisor is dead.
    /// We should start a new viable process.
    pub async fn ping(&mut self) -> Result<(), VirshleError> {
        self.client.ping().await?;
        Ok(())
    }
    /// Create the virtual machine process, but do not boot.
    pub async fn create(&mut self) -> Result<(), VirshleError> {
        self.ping().await?;
        let config = VmConfig::from(self.vm).await?;
        debug!("Pushing config to vmm: {:#?}", config);

        let endpoint = "/vm.create";
        let res = self.client.put::<VmConfig>(endpoint, Some(config)).await?;

        Ok(())
    }
    /// Get info about the vm.
    pub async fn info(&mut self) -> Result<VmInfoResponse, VirshleError> {
        let data = self._info().await?;
        let data: VmInfoResponse = serde_json::from_str(&data)?;
        Ok(data)
    }
    /// Get info about the vm as json.
    pub async fn _info(&mut self) -> Result<String, VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.info";
        let res = self.client.get(endpoint).await?;
        let data = &res.to_string().await?;
        Ok(data.to_owned())
    }
    /// Return vm state,
    /// or the default state if couldn't connect to vm.
    pub async fn state(&mut self) -> Result<VmState, VirshleError> {
        // Safeguard
        let endpoint = "/vm.info";
        let state: VmState;

        match self.client.get(endpoint).await {
            Ok(res) => {
                state = match res.status() {
                    StatusCode::OK => {
                        let data = &res.to_string().await?;
                        let data: VmInfoResponse = serde_json::from_str(&data)?;
                        VmState::from(data.state)
                    }
                    StatusCode::INTERNAL_SERVER_ERROR => VmState::NotCreated,
                    _ => VmState::NotCreated,
                };
            }
            Err(e) => {
                // Endpoint or socket do not exist.
                trace!("{:#?}", e);
                state = VmState::NotCreated
            }
        };
        Ok(state)
    }
    /// Bring the virtual machine up.
    pub async fn boot(&mut self) -> Result<(), VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.boot";
        let res = self.client.put::<()>(endpoint, None).await?;
        if res.status().is_success() {
            let msg = &res.to_string().await?;
            trace!("{}", &msg);
        } else {
            let err_msg = &res.to_string().await?;
            error!("{}", &err_msg);
            let message = "Couldn't boot vm.";
            return Err(LibError::builder()
                .msg(&message)
                .help(&err_msg)
                .build()
                .into());
        }
        Ok(())
    }
    #[tracing::instrument(skip_all)]
    pub async fn pause(&mut self) -> Result<(), VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.pause";
        let res = self.client.put::<()>(endpoint, None).await?;
        trace!("paused vm {}", self.vm.name);
        Ok(())
    }
    /// Delete the virtual machine process.
    pub async fn delete(&mut self) -> Result<(), VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.delete";
        let res = self.client.put::<()>(endpoint, None).await?;
        Ok(())
    }
    pub async fn shutdown(&mut self) -> Result<(), VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.shutdown";
        let res = self.client.put::<()>(endpoint, None).await?;
        Ok(())
    }
    /// Remove a device from Vm.
    pub async fn remove_device(&mut self, device_id: &str) -> Result<(), VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.remove-device";
        let req = VmRemoveDeviceData {
            id: device_id.to_owned(),
        };
        let res = self
            .client
            .put::<VmRemoveDeviceData>(endpoint, Some(req))
            .await?;
        trace!("{:#?}", res);
        Ok(())
    }
    pub async fn add_net(&mut self, net_config: NetConfig) -> Result<(), VirshleError> {
        // Safeguard
        self.ping().await?;
        let endpoint = "/vm.add-net";
        let req = net_config;
        let res = self.client.put::<NetConfig>(endpoint, Some(req)).await?;

        Ok(())
    }
}
