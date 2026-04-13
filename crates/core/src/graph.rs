use crate::types::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Topological sort using Kahn's algorithm
pub fn topo_sort(nodes: &[NotionNode]) -> Vec<NotionNode> {
    if nodes.is_empty() {
        return vec![];
    }

    let id_to_idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    // In-degree: count how many dependencies each node has
    let mut in_degree: Vec<usize> = vec![0; nodes.len()];

    for (i, node) in nodes.iter().enumerate() {
        for dep_id in &node.depends_on_ids {
            if id_to_idx.contains_key(dep_id.as_str()) {
                in_degree[i] += 1;
            }
        }
    }

    // Queue: nodes with no dependencies
    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter_map(|(i, &deg)| if deg == 0 { Some(i) } else { None })
        .collect();

    let mut sorted: Vec<NotionNode> = Vec::with_capacity(nodes.len());

    while let Some(idx) = queue.pop_front() {
        sorted.push(nodes[idx].clone());

        // Find all nodes that depend on this node
        for (j, node) in nodes.iter().enumerate() {
            if node.depends_on_ids.contains(&nodes[idx].id) {
                in_degree[j] -= 1;
                if in_degree[j] == 0 {
                    queue.push_back(j);
                }
            }
        }
    }

    sorted
}

/// Detect nodes that are part of cycles
pub fn detect_cycles(nodes: &[NotionNode]) -> HashSet<String> {
    let sorted = topo_sort(nodes);
    let sorted_ids: HashSet<&str> = sorted.iter().map(|n| n.id.as_str()).collect();

    nodes
        .iter()
        .filter(|n| !sorted_ids.contains(n.id.as_str()))
        .map(|n| n.id.clone())
        .collect()
}

/// Check if any parent in the ancestor chain is archived
fn is_ancestor_archived(id: &str, nodes: &HashMap<String, ComputedNode>) -> bool {
    let mut current_id = id.to_string();
    loop {
        let parent_id = nodes
            .get(&current_id)
            .and_then(|n| n.base.parent_id.clone());

        match parent_id {
            None => return false,
            Some(pid) => {
                if let Some(parent) = nodes.get(&pid) {
                    if parent.base.archived.is_some() {
                        return true;
                    }
                    current_id = pid;
                } else {
                    return false;
                }
            }
        }
    }
}

/// Check if a node and all its children are completed
fn is_node_completed(id: &str, nodes: &HashMap<String, ComputedNode>) -> bool {
    let node = match nodes.get(id) {
        Some(n) => n,
        None => return false,
    };

    match node.node_type {
        NodeType::Leaf => node.base.done,
        NodeType::Container | NodeType::Root => {
            let children: Vec<&ComputedNode> = nodes
                .values()
                .filter(|n| n.base.parent_id.as_deref() == Some(id) && n.base.archived.is_none())
                .collect();

            if children.is_empty() {
                return false;
            }

            children.iter().all(|n| n.status == NodeStatus::Completed)
        }
    }
}

/// Compute status for all nodes based on completion and ancestry
pub fn compute_statuses(nodes: &mut HashMap<String, ComputedNode>) {
    let flat: Vec<NotionNode> = nodes.values().map(|n| n.base.clone()).collect();
    let sorted = topo_sort(&flat);

    for notion_node in &sorted {
        let id = &notion_node.id;

        // Check if archived directly
        if notion_node.archived.is_some() {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Archived;
            }
            continue;
        }

        // Check if any ancestor is archived
        if is_ancestor_archived(id, nodes) {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Archived;
            }
            continue;
        }

        // Root nodes are always active
        if let Some(n) = nodes.get(id) {
            if n.node_type == NodeType::Root {
                if let Some(nm) = nodes.get_mut(id) {
                    nm.status = NodeStatus::Active;
                }
                continue;
            }
        }

        // Check if all dependencies are met
        let deps_met = notion_node.depends_on_ids.iter().all(|dep_id| {
            nodes
                .get(dep_id)
                .map(|dep| {
                    dep.status == NodeStatus::Completed || dep.status == NodeStatus::Archived
                })
                .unwrap_or(true)
        });

        if !deps_met {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Locked;
            }
            continue;
        }

        // Check if completed
        let is_complete = is_node_completed(id, nodes);
        if let Some(n) = nodes.get_mut(id) {
            n.status = if is_complete {
                NodeStatus::Completed
            } else {
                NodeStatus::Active
            };
        }
    }
}

/// Compute progress (0-1) for a node based on children completion
pub fn compute_progress(id: &str, nodes: &HashMap<String, ComputedNode>) -> f64 {
    let node = match nodes.get(id) {
        Some(n) => n,
        None => return 0.0,
    };

    match node.node_type {
        NodeType::Leaf => {
            if node.base.done {
                1.0
            } else {
                0.0
            }
        }
        NodeType::Container | NodeType::Root => {
            let children: Vec<&ComputedNode> = nodes
                .values()
                .filter(|n| {
                    n.base.parent_id.as_deref() == Some(id) && n.base.archived.is_none()
                })
                .collect();

            if children.is_empty() {
                return 0.0;
            }

            let completed = children
                .iter()
                .filter(|n| n.status == NodeStatus::Completed)
                .count();

            completed as f64 / children.len() as f64
        }
    }
}

impl ComputedNode {
    /// Create a ComputedNode from a NotionNode with defaults
    pub fn from_notion(node: NotionNode) -> Self {
        ComputedNode {
            base: node,
            node_type: NodeType::Leaf,
            status: NodeStatus::Active,
            progress: 0.0,
            depth: 0,
            children: vec![],
            dependents: vec![],
            position: Position::default(),
            is_cycle_member: false,
        }
    }
}

/// Build a complete tree from flat notion nodes
pub fn build_tree(nodes: Vec<NotionNode>) -> Vec<ComputedNode> {
    if nodes.is_empty() {
        return vec![];
    }

    // Step 1: Create initial computed nodes
    let mut node_map: HashMap<String, ComputedNode> = nodes
        .iter()
        .map(|n| (n.id.clone(), ComputedNode::from_notion(n.clone())))
        .collect();

    // Step 2: Determine node types
    // has_parent: set of node IDs that have a parent
    let has_parent: HashSet<String> = nodes
        .iter()
        .filter_map(|n| {
            if n.parent_id.is_some() {
                Some(n.id.clone())
            } else {
                None
            }
        })
        .collect();

    // has_children: set of node IDs that are parents
    let has_children: HashSet<String> = nodes
        .iter()
        .filter_map(|n| n.parent_id.as_ref())
        .map(|p| p.clone())
        .collect::<HashSet<_>>();

    for node in node_map.values_mut() {
        node.node_type = if !has_parent.contains(&node.base.id) {
            NodeType::Root
        } else if has_children.contains(&node.base.id) {
            NodeType::Container
        } else {
            NodeType::Leaf
        };
    }

    // Step 3: Assign depths (BFS from roots)
    let mut depths: HashMap<String, u32> = HashMap::new();
    let roots: Vec<String> = node_map
        .values()
        .filter(|n| n.node_type == NodeType::Root)
        .map(|n| n.base.id.clone())
        .collect();

    let mut queue = VecDeque::new();
    for root_id in roots {
        depths.insert(root_id.clone(), 0);
        queue.push_back(root_id);
    }

    while let Some(current_id) = queue.pop_front() {
        let current_depth = depths[&current_id];
        for child_node in nodes.iter() {
            if child_node.parent_id.as_ref() == Some(&current_id) {
                if !depths.contains_key(&child_node.id) {
                    depths.insert(child_node.id.clone(), current_depth + 1);
                    queue.push_back(child_node.id.clone());
                }
            }
        }
    }

    for (id, depth) in depths {
        if let Some(node) = node_map.get_mut(&id) {
            node.depth = depth;
        }
    }

    // Step 4: Detect cycles
    let cycle_members = detect_cycles(&nodes);
    for id in cycle_members {
        if let Some(node) = node_map.get_mut(&id) {
            node.is_cycle_member = true;
        }
    }

    // Step 5: Compute statuses
    compute_statuses(&mut node_map);

    // Step 6: Compute progress
    let ids: Vec<String> = node_map.keys().cloned().collect();
    for id in ids {
        let progress = compute_progress(&id, &node_map);
        if let Some(node) = node_map.get_mut(&id) {
            node.progress = progress;
        }
    }

    // Step 7: Build recursive tree structure
    let root_ids: Vec<String> = node_map
        .values()
        .filter(|n| n.node_type == NodeType::Root)
        .map(|n| n.base.id.clone())
        .collect();

    let mut roots = Vec::new();
    for root_id in root_ids {
        if let Some(root) = build_node_recursive(&root_id, &node_map, &nodes) {
            roots.push(root);
        }
    }

    roots
}

/// Recursively build a node with its children
fn build_node_recursive(
    id: &str,
    node_map: &HashMap<String, ComputedNode>,
    all_nodes: &[NotionNode],
) -> Option<ComputedNode> {
    let mut node = node_map.get(id)?.clone();

    // Find children
    for child_node in all_nodes {
        if let Some(parent_id) = &child_node.parent_id {
            if parent_id == id {
                if let Some(child) = build_node_recursive(&child_node.id, node_map, all_nodes) {
                    node.children.push(child);
                }
            }
        }
    }

    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, parent_id: Option<&str>, done: bool) -> NotionNode {
        NotionNode {
            id: id.to_string(),
            name: id.to_string(),
            icon: None,
            description: String::new(),
            why: String::new(),
            criteria: String::new(),
            parent_id: parent_id.map(|s| s.to_string()),
            depends_on_ids: vec![],
            done,
            archived: None,
            pinned: false,
            badge: None,
            color: None,
            due: None,
            time_range: None,
            resources: vec![],
            created_time: String::new(),
        }
    }

    #[test]
    fn test_topo_sort_linear() {
        let nodes = vec![
            create_test_node("a", None, false),
            create_test_node("b", None, false),
        ];
        let sorted = topo_sort(&nodes);
        assert_eq!(sorted.len(), 2);
    }

    #[test]
    fn test_detect_cycles_no_cycle() {
        let nodes = vec![
            create_test_node("a", None, false),
            create_test_node("b", None, false),
        ];
        let cycles = detect_cycles(&nodes);
        assert_eq!(cycles.len(), 0);
    }

    #[test]
    fn test_build_tree_single_node() {
        let nodes = vec![create_test_node("root", None, false)];
        let tree = build_tree(nodes);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].node_type, NodeType::Root);
        assert_eq!(tree[0].depth, 0);
    }

    #[test]
    fn test_build_tree_parent_child() {
        let nodes = vec![
            create_test_node("root", None, false),
            create_test_node("child", Some("root"), false),
        ];
        let tree = build_tree(nodes);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].node_type, NodeType::Root);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].node_type, NodeType::Leaf);
    }
}
