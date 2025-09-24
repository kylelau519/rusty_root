//! ROOT Tree handling

use crate::error::{Result, RootError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a branch in a ROOT tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub data_type: String,
    pub entries: u64,
    pub data: Vec<f64>, // Simplified - in reality would be generic
}

/// Represents a ROOT TTree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub name: String,
    pub title: String,
    pub entries: u64,
    pub branches: HashMap<String, Branch>,
}

impl Tree {
    /// Create a new empty tree
    pub fn new(name: String, title: String) -> Self {
        Self {
            name,
            title,
            entries: 0,
            branches: HashMap::new(),
        }
    }
    
    /// Add a branch to the tree
    pub fn add_branch(&mut self, branch: Branch) {
        self.branches.insert(branch.name.clone(), branch);
    }
    
    /// Get a branch by name
    pub fn get_branch(&self, name: &str) -> Option<&Branch> {
        self.branches.get(name)
    }
    
    /// Get all branch names
    pub fn branch_names(&self) -> Vec<String> {
        self.branches.keys().cloned().collect()
    }
    
    /// Get tree summary information
    pub fn summary(&self) -> String {
        format!(
            "Tree: {} ({})\nEntries: {}\nBranches: {}",
            self.name,
            self.title,
            self.entries,
            self.branches.len()
        )
    }
    
    /// Create a sample tree for demonstration
    pub fn create_sample() -> Self {
        let mut tree = Tree::new("sample_tree".to_string(), "Sample Tree for Demo".to_string());
        
        // Create sample branches with mock data
        let x_branch = Branch {
            name: "x".to_string(),
            data_type: "Double_t".to_string(),
            entries: 100,
            data: (0..100).map(|i| i as f64 * 0.1).collect(),
        };
        
        let y_branch = Branch {
            name: "y".to_string(),
            data_type: "Double_t".to_string(),
            entries: 100,
            data: (0..100).map(|i| (i as f64 * 0.1).sin()).collect(),
        };
        
        let energy_branch = Branch {
            name: "energy".to_string(),
            data_type: "Float_t".to_string(),
            entries: 100,
            data: (0..100).map(|i| 100.0 + (i as f64).sqrt() * 10.0).collect(),
        };
        
        tree.entries = 100;
        tree.add_branch(x_branch);
        tree.add_branch(y_branch);
        tree.add_branch(energy_branch);
        
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree() {
        let tree = Tree::new("test".to_string(), "Test Tree".to_string());
        assert_eq!(tree.name, "test");
        assert_eq!(tree.title, "Test Tree");
        assert_eq!(tree.entries, 0);
        assert!(tree.branches.is_empty());
    }

    #[test]
    fn test_add_branch() {
        let mut tree = Tree::new("test".to_string(), "Test Tree".to_string());
        let branch = Branch {
            name: "test_branch".to_string(),
            data_type: "Double_t".to_string(),
            entries: 10,
            data: vec![1.0, 2.0, 3.0],
        };
        tree.add_branch(branch);
        
        assert_eq!(tree.branches.len(), 1);
        assert!(tree.get_branch("test_branch").is_some());
    }

    #[test]
    fn test_sample_tree() {
        let tree = Tree::create_sample();
        assert_eq!(tree.name, "sample_tree");
        assert_eq!(tree.entries, 100);
        assert_eq!(tree.branches.len(), 3);
        assert!(tree.get_branch("x").is_some());
        assert!(tree.get_branch("y").is_some());
        assert!(tree.get_branch("energy").is_some());
    }
}