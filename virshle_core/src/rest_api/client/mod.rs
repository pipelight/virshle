mod methods;

use bon::bon;
use crate::config::{Config, Node};

// Error handling
use miette::{IntoDiagnostic, Result};
use tracing::{error, info, trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Client {
    nodes: Vec<Node>,
}
impl Client {
    pub fn new()-> Result<Self> {
        let nodes = Config::get()?.nodes()?;
        let client = Client {
            nodes,
        };
        Ok(client)
    }
}
pub struct VmMethods;

#[bon]
impl VmMethods {

    /// Start a virtual machine on a node.
    #[builder(finish_fn = get)]
    pub async fn start(
        // GetVmArgs
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,
        args: GetVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<VmTable, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] starting a vm on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vm = rest
            .put("/vm/start", Some((args, user_data)))
            .await?
            .to_value()
            .await?;
        conn.close();

        info!("[end] started vm {:#?} on node {:#?}", vm.name, node.name);

        Ok(VmTable::from(&vm).await?)
    }

}
    




impl Client {
    pub fn all_node() -> {

    }
    pub fn one_node() -> {
    }
}

#[bon]
impl NodeMethods<'_> {
    #[builder(finish_fn = get)]
    pub fn all(&self) -> VirshleClientMethods {

    }
    pub fn one(&self) -> VirshleClientMethods {}
    pub fn api(&self) -> VirshleClientMethods {
        let conn = Connection::from(self.vm);
        let mut rest = RestClient::from(node);
        rest.base_url("/api/v1");
        rest.ping_url(&format!("{}{}", "/api/v1", "/node/ping"));
        VirshleClientMethods {
            vm: self.vm,
            client: rest,
        }
    }
}
