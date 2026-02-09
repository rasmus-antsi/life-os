use std::path::{Path, PathBuf};

use crate::spec::Node;

pub fn check_tree(base: &Path, nodes: &[Node], missing: &mut Vec<PathBuf>) {
    for node in nodes {
        let path = base.join(&node.path);
        if !path.exists() {
            missing.push(path.clone());
        }

        if !node.children.is_empty() {
            check_tree(&path, &node.children, missing);
        }
    }
}
