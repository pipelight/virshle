use super::{Node, VirshleConfig, Vm, VmTemplate};

// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, TomlError, VirshleError, WrapError};

impl VirshleConfig {
    pub fn get_vm_templates(&self) -> Result<HashMap<String, VmTemplate>, VirshleError> {
        let mut hashmap = HashMap::new();
        if let Some(template) = &self.template {
            if let Some(vm) = &template.vm {
                hashmap = vm.iter().map(|e| (e.name.clone(), e.to_owned())).collect();
            }
        }
        Ok(hashmap)
    }
    pub fn get_template(&self, name: &str) -> Result<VmTemplate, VirshleError> {
        let templates = self.get_vm_templates()?;
        let res = templates.get(name);
        match res {
            Some(res) => Ok(res.to_owned()),
            None => {
                let message = format!("Couldn't find template {:#?}", name);
                let templates_name = templates
                    .iter()
                    .map(|e| e.0.to_owned())
                    .collect::<Vec<String>>()
                    .join(",");
                let help = format!("Available templates are:\n[{templates_name}]");
                let err = LibError::builder().msg(&message).help(&help).build();
                Err(err.into())
            }
        }
    }
    /*
     * Returns nodes defined in configuration,
     * plus the default local node.
     */
    pub fn get_nodes(&self) -> Result<Vec<Node>, VirshleError> {
        let mut nodes: Vec<Node> = vec![Node::default()];
        if let Some(node) = &self.node {
            nodes.extend(node.to_owned());
        }
        Ok(nodes)
    }
    /*
     * Returns node with name.
     */
    pub fn get_node_by_name(&self, name: &str) -> Result<Node, VirshleError> {
        let nodes: Vec<Node> = self.get_nodes()?;
        let filtered_nodes: Vec<Node> = self
            .get_nodes()?
            .iter()
            .filter(|e| e.name == name)
            .map(|e| e.to_owned())
            .collect();

        let node = filtered_nodes.first();
        match node {
            Some(node) => Ok(node.to_owned()),
            None => {
                let node_names: Vec<String> = nodes.iter().map(|e| e.name.to_owned()).collect();
                let node_names: String = node_names.join("\t\n");
                let message = format!("couldn't find node with name: {:#?}", name);
                let help = format!("Available nodes are: \n");
                let err = LibError::builder().msg(&message).help(&help).build();
                return Err(err.into());
            }
        }
    }
}
