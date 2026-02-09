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

#[cfg(test)]
mod tests {
    use super::check_tree;
    use crate::spec::Node;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn check_tree_collects_missing_paths() {
        let dir = tempdir().expect("tempdir");
        let base = dir.path();

        fs::create_dir_all(base.join("exists")).expect("create dir");

        let nodes = vec![
            Node {
                path: "exists".to_string(),
                children: vec![Node {
                    path: "child-missing".to_string(),
                    children: vec![],
                }],
            },
            Node {
                path: "missing".to_string(),
                children: vec![],
            },
        ];

        let mut missing = Vec::new();
        check_tree(base, &nodes, &mut missing);

        missing.sort();
        let expected = vec![base.join("exists/child-missing"), base.join("missing")];
        assert_eq!(missing, expected);
    }
}
