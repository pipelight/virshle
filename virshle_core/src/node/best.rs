use std::cmp::Ordering;

use std::collections::HashMap;
use virshle_network::connection::ConnectionState;

use crate::node::{Node, NodeInfo};

// Random
use rand::prelude::IndexedRandom;

// Error Handling
use miette::Result;
use tracing::warn;
use virshle_error::{LibError, VirshleError};

// impl Node {
//     // Get random non-saturated node.
//     pub async fn get_by_random() -> Result<Self, VirshleError> {
//         let nodes: HashMap<Node, (ConnectionState, Option<NodeInfo>)> =
//             client::node::get_info_all().await?;
//
//         let mut ref_vec: Vec<&Node> = vec![];
//         for (node, (state, info)) in &nodes {
//             if let Some(info) = info {
//                 // Remove saturated nodes
//                 if info.get_saturation_index().await? < 1.0 {
//                     ref_vec.push(&node)
//                 }
//             }
//         }
//         match ref_vec.choose(&mut rand::rng()) {
//             Some(node) => Ok(node.to_owned().to_owned()),
//             None => Err(LibError::builder()
//                 .msg("Couldn't get a proper node.")
//                 .help("Nodes unreachable or saturated!")
//                 .build()
//                 .into()),
//         }
//     }
//
//     // Get random non-saturated node with weight.
//     pub async fn get_by_load_balance() -> Result<Self, VirshleError> {
//         let nodes: HashMap<Node, (ConnectionState, Option<NodeInfo>)> =
//             client::node::get_info_all().await?;
//
//         let mut ref_vec: Vec<&Node> = vec![];
//         for (node, (state, info)) in &nodes {
//             if let Some(info) = info {
//                 // Remove saturated nodes
//                 if info.get_saturation_index().await? < 1.0 {
//                     let weighted_vec: Vec<&Node>;
//                     // Add weight to node
//                     if let Some(weight) = node.weight {
//                         weighted_vec = std::iter::repeat_n(node, weight as usize).collect();
//                     } else {
//                         weighted_vec = vec![&node];
//                     }
//                     ref_vec.extend(weighted_vec);
//                 }
//             }
//         }
//         match ref_vec.choose(&mut rand::rng()) {
//             Some(node_ref) => Ok(node_ref.to_owned().to_owned()),
//             None => Err(LibError::builder()
//                 .msg("Couldn't get a proper node.")
//                 .help("Nodes unreachable or saturated!")
//                 .build()
//                 .into()),
//         }
//     }
//
//     // Get random non-saturated node by round-robin.
//     pub async fn get_by_saturation_index() -> Result<Self, VirshleError> {
//         let nodes: HashMap<Node, (ConnectionState, Option<NodeInfo>)> =
//             client::node::get_info_all().await?;
//
//         let mut ref_vec: Vec<(f64, &Node)> = vec![];
//         for (node, (state, info)) in &nodes {
//             if let Some(info) = info {
//                 // Remove saturated nodes
//                 if info.get_saturation_index().await? < 1.0 {
//                     let s_index = info.get_saturation_index().await?;
//                     ref_vec.push((s_index, &node));
//                 }
//             }
//         }
//         // Find lowest saturation index.
//         ref_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
//         ref_vec.first();
//
//         match ref_vec.first() {
//             Some((_, node)) => Ok(node.to_owned().to_owned()),
//             None => Err(LibError::builder()
//                 .msg("Couldn't get a proper node.")
//                 .help("Nodes unreachable or saturated!")
//                 .build()
//                 .into()),
//         }
//     }
// }
