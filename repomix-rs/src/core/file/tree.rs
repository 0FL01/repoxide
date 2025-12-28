//! Directory tree generation
//!
//! Provides ASCII tree representation of directory structure.

use std::collections::BTreeMap;

/// Tree node representing a file or directory
#[derive(Debug, Clone)]
struct TreeNode {
    name: String,
    is_directory: bool,
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn new(name: &str, is_directory: bool) -> Self {
        Self {
            name: name.to_string(),
            is_directory,
            children: BTreeMap::new(),
        }
    }
}

/// Build a tree from a list of file paths
fn build_tree(files: &[String], empty_dirs: &[String]) -> TreeNode {
    let mut root = TreeNode::new("root", true);
    
    // Add files
    for file in files {
        add_path_to_tree(&mut root, file, false);
    }
    
    // Add empty directories
    for dir in empty_dirs {
        add_path_to_tree(&mut root, dir, true);
    }
    
    root
}

/// Add a path to the tree
fn add_path_to_tree(root: &mut TreeNode, path: &str, is_directory: bool) {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current = root;
    
    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        let is_dir = !is_last || is_directory;
        
        if !current.children.contains_key(*part) {
            current.children.insert(part.to_string(), TreeNode::new(part, is_dir));
        }
        
        current = current.children.get_mut(*part).unwrap();
    }
}

/// Convert tree to string representation
fn tree_to_string(node: &TreeNode, prefix: &str) -> String {
    let mut result = String::new();
    
    // Sort: directories first, then files, both alphabetically
    let mut items: Vec<_> = node.children.iter().collect();
    items.sort_by(|a, b| {
        match (a.1.is_directory, b.1.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(b.0),
        }
    });
    
    for (_, child) in items {
        let suffix = if child.is_directory { "/" } else { "" };
        result.push_str(&format!("{}{}{}\n", prefix, child.name, suffix));
        
        if child.is_directory {
            result.push_str(&tree_to_string(child, &format!("{}  ", prefix)));
        }
    }
    
    result
}

/// Convert tree to string with line counts for files
fn tree_to_string_with_line_counts(
    node: &TreeNode,
    line_counts: &std::collections::HashMap<String, usize>,
    prefix: &str,
    current_path: &str,
) -> String {
    let mut result = String::new();
    
    // Sort: directories first, then files, both alphabetically
    let mut items: Vec<_> = node.children.iter().collect();
    items.sort_by(|a, b| {
        match (a.1.is_directory, b.1.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(b.0),
        }
    });
    
    for (name, child) in items {
        let child_path = if current_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", current_path, name)
        };
        
        if child.is_directory {
            result.push_str(&format!("{}{}/\n", prefix, child.name));
            result.push_str(&tree_to_string_with_line_counts(
                child,
                line_counts,
                &format!("{}  ", prefix),
                &child_path,
            ));
        } else {
            let line_count_suffix = line_counts
                .get(&child_path)
                .map(|c| format!(" ({} lines)", c))
                .unwrap_or_default();
            result.push_str(&format!("{}{}{}\n", prefix, child.name, line_count_suffix));
        }
    }
    
    result
}

/// Generate ASCII tree representation of directory structure
///
/// # Arguments
/// * `files` - List of file paths relative to root
/// * `empty_dirs` - Optional list of empty directory paths
///
/// # Returns
/// ASCII tree string representation
pub fn generate_tree(files: &[String], empty_dirs: &[String]) -> String {
    let tree = build_tree(files, empty_dirs);
    tree_to_string(&tree, "").trim_end().to_string()
}

/// Generate ASCII tree with line counts for each file
///
/// # Arguments
/// * `files` - List of file paths relative to root
/// * `line_counts` - Map of file paths to line counts
/// * `empty_dirs` - Optional list of empty directory paths
///
/// # Returns
/// ASCII tree string representation with line counts
pub fn generate_tree_with_line_counts(
    files: &[String],
    line_counts: &std::collections::HashMap<String, usize>,
    empty_dirs: &[String],
) -> String {
    let tree = build_tree(files, empty_dirs);
    tree_to_string_with_line_counts(&tree, line_counts, "", "")
        .trim_end()
        .to_string()
}

/// Count lines in a string
pub fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.lines().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tree_simple() {
        let files = vec![
            "main.rs".to_string(),
            "lib.rs".to_string(),
        ];
        
        let tree = generate_tree(&files, &[]);
        
        assert!(tree.contains("lib.rs"));
        assert!(tree.contains("main.rs"));
    }

    #[test]
    fn test_generate_tree_nested() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/utils/mod.rs".to_string(),
            "Cargo.toml".to_string(),
        ];
        
        let tree = generate_tree(&files, &[]);
        
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
        assert!(tree.contains("utils/"));
        assert!(tree.contains("Cargo.toml"));
    }

    #[test]
    fn test_generate_tree_sort_order() {
        let files = vec![
            "z.txt".to_string(),
            "src/b.rs".to_string(),
            "a.txt".to_string(),
            "src/a.rs".to_string(),
        ];
        
        let tree = generate_tree(&files, &[]);
        let lines: Vec<&str> = tree.lines().collect();
        
        // Directory should come first
        assert!(lines[0].contains("src/"));
        // Then files alphabetically
        assert!(lines.iter().position(|l| l.contains("a.txt")).unwrap() < lines.iter().position(|l| l.contains("z.txt")).unwrap());
    }

    #[test]
    fn test_generate_tree_with_empty_dirs() {
        let files = vec!["src/main.rs".to_string()];
        let empty_dirs = vec!["empty_dir".to_string()];
        
        let tree = generate_tree(&files, &empty_dirs);
        
        assert!(tree.contains("empty_dir/"));
        assert!(tree.contains("src/"));
    }

    #[test]
    fn test_generate_tree_with_line_counts() {
        let files = vec![
            "main.rs".to_string(),
            "lib.rs".to_string(),
        ];
        
        let mut line_counts = std::collections::HashMap::new();
        line_counts.insert("main.rs".to_string(), 10);
        line_counts.insert("lib.rs".to_string(), 25);
        
        let tree = generate_tree_with_line_counts(&files, &line_counts, &[]);
        
        assert!(tree.contains("main.rs (10 lines)"));
        assert!(tree.contains("lib.rs (25 lines)"));
    }

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("one"), 1);
        assert_eq!(count_lines("one\ntwo"), 2);
        assert_eq!(count_lines("one\ntwo\nthree"), 3);
    }
}
