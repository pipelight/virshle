use super::{Node, VirshleConfig, Vm, VmTemplate};

// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error Handling
use miette::{Error, IntoDiagnostic, Result};
use tracing::info;
use virshle_error::{CastError, LibError, TomlError, VirshleError, WrapError};

impl VirshleConfig {
    pub fn get_templates(&self) -> Result<Vec<VmTemplate>, VirshleError> {
        if let Some(template) = &self.template {
            if let Some(vm) = &template.vm {
                return Ok(vm.to_owned());
            }
        }
        Ok(vec![])
    }
    pub fn get_template(&self, name: &str) -> Result<VmTemplate, VirshleError> {
        let templates = self.get_templates()?;
        let res = templates.iter().find(|e| e.name == name);
        match res {
            Some(res) => Ok(res.to_owned()),
            None => {
                let message = format!("Couldn't find template {:#?}", name);
                let templates_name = templates
                    .iter()
                    .map(|e| e.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(",");
                let help = format!("Available templates are:\n[{templates_name}]");
                let err = LibError::builder().msg(&message).help(&help).build();
                Err(err.into())
            }
        }
    }
}
