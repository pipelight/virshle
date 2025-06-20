use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};

use bon::{bon, Builder};
// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

/*
* The different type of action you can execute with ovs-vsctl.
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OvsAction {
    Create,
    Delete,
    Get,
}
/*
* The different type of network interface in ovs.
*/
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OvsInterfaceType {
    #[default]
    System,
    Internal,
    Patch,
    DpdkVhostUserClient,
    Tap,
}
impl ToString for OvsInterfaceType {
    fn to_string(&self) -> String {
        match self {
            Self::System => "system".to_string(),
            Self::Internal => "internal".to_string(),
            Self::Patch => "patch".to_string(),
            Self::DpdkVhostUserClient => "dpdkvhostuserclient".to_string(),
            Self::Tap => "tap".to_string(),
        }
    }
}

/*
* The different type of bridges.
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OvsBridgeType {
    // managed by system (to be used for tap)
    System,
    // managed by ovs (to be used for dpdk)
    Netdev,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OvsBridgeBuilder {
    bridge: String,
    action: OvsAction,
    _type: OvsBridgeType,
    // Final command
    stdin: String,
}
impl OvsBridgeBuilder {
    pub fn create(&mut self) -> Self {
        self.action = OvsAction::Create;
        self.to_owned()
    }
    pub fn delete(&mut self) -> Self {
        self.action = OvsAction::Delete;
        self.to_owned()
    }
    pub fn get(&mut self) -> &mut Self {
        self.action = OvsAction::Get;
        self
    }
    pub fn _type(&mut self, _type: OvsBridgeType) -> Self {
        self._type = _type;
        self.to_owned()
    }
    pub fn build(&mut self) -> Self {
        let mut cmd: Vec<String> = vec![];

        #[cfg(debug_assertions)]
        cmd.push("sudo ovs-vsctl".to_string());
        #[cfg(not(debug_assertions))]
        cmd.push("ovs-vsctl".to_string());

        match self.action {
            OvsAction::Get => {
                cmd.push(format!("--if-exists list bridge {}", self.bridge));
            }
            OvsAction::Create => {
                cmd.push(format!("--may-exist add-br {}", self.bridge));
            }
            OvsAction::Delete => {
                cmd.push(format!("--if-exists del-br {}", self.bridge));
            }
        };

        let stdin = cmd.join(" -- ");
        self.stdin = stdin;

        self.to_owned()
    }
    pub fn exec(&self) -> Result<(), VirshleError> {
        let mut proc = Process::new();
        let res = proc.stdin(&self.stdin).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed:";
            let help = format!("{}\n{} ", &res.io.stdin.unwrap(), stderr);
            return Err(LibError::builder().msg(message).help(&help).build().into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OvsInterfaceBuilder {
    bridge: Option<String>,
    interface: String,
    action: OvsAction,
    _type: OvsInterfaceType,

    // For peer type interface
    peer: Option<String>,
    // For dpdkvhostuser* type interfaces
    socket_path: Option<String>,

    // Final command
    stdin: String,
}
impl OvsInterfaceBuilder {
    pub fn bridge(&mut self, name: &str) -> &mut Self {
        self.bridge = Some(name.to_string());
        self
    }
    pub fn create(&mut self) -> &mut Self {
        self.action = OvsAction::Create;
        self
    }
    pub fn delete(&mut self) -> &mut Self {
        self.action = OvsAction::Delete;
        self
    }
    pub fn get(&mut self) -> &mut Self {
        self.action = OvsAction::Get;
        self
    }
    pub fn _type(&mut self, _type: OvsInterfaceType) -> &mut Self {
        self._type = _type;
        self
    }
    /*
     * For patch interface only.
     */
    pub fn peer(&mut self, name: &str) -> &mut Self {
        self.peer = Some(name.to_string());
        self
    }
    /*
     * For dpdkvhostuser* type interfaces
     */
    pub fn socket_path(&mut self, path: &str) -> &mut Self {
        self.socket_path = Some(path.to_string());
        self
    }

    pub fn build(&mut self) -> Self {
        let mut cmd: Vec<String> = vec![];

        #[cfg(debug_assertions)]
        cmd.push("sudo ovs-vsctl".to_string());
        #[cfg(not(debug_assertions))]
        cmd.push("ovs-vsctl".to_string());

        match self.action {
            OvsAction::Get => {
                cmd.push(format!("--if-exists list port {}", self.interface));
            }
            OvsAction::Create => {
                if let Some(bridge) = &self.bridge {
                    let iface = &self.interface;
                    let _type = &self._type.to_string();
                    cmd.push(format!("--may-exist add-port {bridge} {iface}"));

                    if self._type == OvsInterfaceType::Patch {
                        if let Some(peer) = &self.peer {
                            cmd.push(format!(
                                "set interface {iface} type={_type} options:peer={peer}"
                            ));
                        }
                    } else if self._type == OvsInterfaceType::DpdkVhostUserClient {
                        if let Some(path) = &self.socket_path {
                            cmd.push(format!(
                                "set interface {iface} type={_type} options:vhost-server-path={path}"
                            ));
                        }
                    } else {
                        cmd.push(format!("set interface {iface} type={_type}"));
                    }
                }
            }
            OvsAction::Delete => {
                cmd.push(format!("--if-exists del-port {}", self.interface));
            }
        };

        let stdin = cmd.join(" -- ");
        self.stdin = stdin;

        self.to_owned()
    }
    pub fn exec(&self) -> Result<(), VirshleError> {
        let mut proc = Process::new();
        let res = proc.stdin(&self.stdin).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed:";
            let help = format!("{}\n{} ", &res.io.stdin.unwrap(), stderr);
            return Err(LibError::builder().msg(message).help(&help).build().into());
        }
        Ok(())
    }
}

/*
* A request builder to ease the burden of working with ovs.
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OvsRequest {
    bridge: Option<OvsBridgeBuilder>,
    interface: Option<OvsInterfaceBuilder>,
    // Final command
    stdin: String,
}

impl OvsRequest {
    pub fn bridge(name: &str) -> OvsBridgeBuilder {
        OvsBridgeBuilder {
            bridge: name.to_string(),
            _type: OvsBridgeType::System,
            action: OvsAction::Get,
            stdin: "".to_string(),
        }
    }
    pub fn interface(name: &str) -> OvsInterfaceBuilder {
        OvsInterfaceBuilder {
            bridge: None,
            interface: name.to_string(),
            action: OvsAction::Get,
            _type: OvsInterfaceType::Internal,
            stdin: "".to_string(),
            peer: None,
            socket_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    // Port/Interface
    #[test]
    fn list_ovs_port() -> Result<()> {
        let req = OvsRequest::interface("br0p1").get().build();
        assert_str_eq!("sudo ovs-vsctl -- --if-exists list port br0p1", req.stdin,);
        Ok(())
    }
    #[test]
    fn create_ovs_port() -> Result<()> {
        let req = OvsRequest::interface("br0p1")
            .bridge("br0")
            .create()
            .build();
        assert_str_eq!(
            "sudo ovs-vsctl -- --may-exist add-port br0 br0p1 \
            -- set interface br0p1 type=internal",
            req.stdin,
        );
        Ok(())
    }
    #[test]
    fn delete_ovs_port() -> Result<()> {
        let req = OvsRequest::interface("br0p1").delete().build();
        assert_str_eq!("sudo ovs-vsctl -- --if-exists del-port br0p1", req.stdin,);
        Ok(())
    }
    // Bridges/Switches
    #[test]
    fn list_ovs_bridge() -> Result<()> {
        let req = OvsRequest::bridge("br0").get().build();
        assert_str_eq!("sudo ovs-vsctl -- --if-exists list bridge br0", req.stdin,);
        Ok(())
    }
    #[test]
    fn create_ovs_bridge() -> Result<()> {
        let req = OvsRequest::bridge("br0").create().build();
        assert_str_eq!("sudo ovs-vsctl -- --may-exist add-br br0", req.stdin,);
        Ok(())
    }
    #[test]
    fn delete_ovs_bridge() -> Result<()> {
        let req = OvsRequest::bridge("br0").delete().build();
        assert_str_eq!("sudo ovs-vsctl -- --if-exists del-br br0", req.stdin,);
        Ok(())
    }
}
