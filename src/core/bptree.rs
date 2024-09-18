use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct BPTree {
    root: Option<Box<Node>>,
    order: usize,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Node {
    keys: Vec<i32>,
    children: Vec<Option<Box<Node>>>,
    is_leaf: bool,
}

impl BPTree {
    pub fn new(order: usize) -> Self {
        BPTree { root: None, order }
    }

    pub fn insert(&mut self, key: i32) {
        // Implement insertion logic here
        let root = self.root.take();
        let (new_root, _) = self.insert_into_node(root, key);
        self.root = new_root;
    }

    pub fn search(&self, key: i32) -> Option<i32> {
        // Implement search logic here
        let mut current = self.root.clone();
        while let Some(node) = current {
            let index = node.keys.iter().find(|&&k| k >= key).map_or_else(
                || node.keys.len(),
                |k| node.keys.iter().take_while(|&&r| r < *k).count(),
            );
            if index < node.keys.len() && node.keys[index] == key {
                return Some(key);
            }
            if node.is_leaf {
                return None;
            }
            current = node.children[index].clone();
        }
        None
    }

    pub fn delete(&mut self, key: i32) {
        let root = self.root.take();
        let (new_root, _) = self.delete_from_node(root, key);
        self.root = new_root;
    }

    pub fn traverse(&self) -> Vec<i32> {
        let mut keys = Vec::new();
        self.traverse_node(&self.root, &mut keys);
        keys
    }

    fn traverse_node(&self, node: &Option<Box<Node>>, keys: &mut Vec<i32>) {
        if let Some(node) = node {
            for key in &node.keys {
                keys.push(*key);
            }
            if node.is_leaf {
                return;
            }
            for child in &node.children {
                self.traverse_node(child, keys);
            }
        }
    }

    fn insert_into_node(&mut self, node: Option<Box<Node>>, key: i32) -> (Option<Box<Node>>, bool) {
        let mut node = node;
        let mut key_inserted = false;

        if let Some(ref mut node) = node {
            let index = node.keys.iter().find(|&&k| k >= key).map_or_else(
                || node.keys.len(),
                |k| node.keys.iter().take_while(|&&r| r < *k).count(),
            );
            if index < node.keys.len() && node.keys[index] == key {
                key_inserted = true;
            } else if node.is_leaf {
                node.keys.insert(index, key);
                key_inserted = true;
            } else {
                let (child, inserted) = self.insert_into_node(node.children[index].clone(), key);
                node.children[index] = child;
                key_inserted = inserted;
                if inserted && node.keys.len() == self.order {
                    self.split_child(node, index);
                }
            }
        } else {
            node = Some(Box::new(Node {
                keys: vec![key],
                children: vec![None; self.order],
                is_leaf: true,
            }));
            key_inserted = true;
        }

        (node, key_inserted)
    }

    fn split_child(&mut self, node: &mut Box<Node>, index: usize) {
        let new_node = Box::new(Node {
            keys: node.keys.split_off(index),
            children: node.children.split_off(index),
            is_leaf: node.is_leaf,
        });
        node.keys.insert(
            index,
            *new_node
                .keys
                .first()
                .expect("New node must have at least one key"),
        );
        node.children.insert(index, Some(new_node));
    }

    fn delete_from_node(&mut self, node: Option<Box<Node>>, key: i32) -> (Option<Box<Node>>, bool) {
        let mut node = node;
        let mut key_deleted = false;

        if let Some(ref mut node) = node {
            let index = node.keys.iter().find(|&&k| k >= key).map_or_else(
                || node.keys.len(),
                |k| node.keys.iter().take_while(|&&r| r < *k).count(),
            );
            if index < node.keys.len() && node.keys[index] == key {
                if node.is_leaf {
                    node.keys.remove(index);
                    key_deleted = true;
                } else {
                    let (child, deleted) = self.delete_from_node(node.children[index].clone(), key);
                    node.children[index] = child;
                    key_deleted = deleted;
                    if deleted && node.keys.len() < self.order {
                        self.merge_child(node, index);
                    }
                }
            }
        }

        (node, key_deleted)
    }

    fn merge_child(&mut self, node: &mut Box<Node>, index: usize) {
        let left_child = node.children[index].clone();
        let right_child = node.children[index + 1].clone();
        let key = node.keys[index];

        if let Some(mut left_child) = left_child {
            if let Some(right_child) = right_child {
                left_child.keys.push(key);
                left_child.keys.extend_from_slice(&right_child.keys);
                left_child.children.extend_from_slice(&right_child.children);
                node.keys.remove(index);
                node.children.remove(index);
            }
        }
    }
}
