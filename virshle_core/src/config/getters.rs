use super::{Node, VirshleConfig, VmTemplate};

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
                let err = LibError::new(&message, &help);
                Err(err.into())
            }
        }
    }
    /*
     * Returns nodes defined in configuration,
     * plus the default local node.
     */
    pub fn get_nodes(&self) -> Result<Vec<Node>, VirshleError> {
        // Add a default local node in case it doesn't exists.
        let mut nodes: Vec<Node> = vec![Node::default()];
        if let Some(node) = &self.node {
            nodes.extend(node.to_owned());
        }

        Ok(nodes)
    }
}
