# Algorithm Porting Guide — TypeScript → Rust
> `lib/graph.ts` (868 lines) → `crates/core/src/graph.rs`

---

## Overview

All graph algorithms live in `crates/core/src/graph.rs`. They operate on flat `Vec<NotionNode>` and produce `Vec<ComputedNode>`. These functions must compile for both native (backend) and WASM (frontend, if graph logic is needed client-side).

---

## 1. `buildTree`

**TypeScript**: `buildTree(nodes: NotionNode[]): ComputedNode[]`

**Purpose**: Converts flat Notion array → rich tree with computed fields.

```rust
pub fn build_tree(nodes: Vec<NotionNode>) -> Vec<ComputedNode> {
    if nodes.is_empty() {
        return vec![];
    }
    
    // Step 1: Initialize working map (id → mutable ComputedNode)
    let mut node_map: HashMap<String, ComputedNode> = nodes
        .iter()
        .map(|n| (n.id.clone(), ComputedNode::from_notion(n.clone())))
        .collect();
    
    // Step 2: Build parent-child relationships
    // Collect children assignments to avoid borrow conflicts
    let child_assignments: Vec<(String, String)> = nodes.iter()
        .filter_map(|n| n.parent_id.as_ref().map(|p| (p.clone(), n.id.clone())))
        .collect();
    
    for (parent_id, child_id) in &child_assignments {
        if node_map.contains_key(parent_id) {
            if let Some(parent) = node_map.get_mut(parent_id) {
                parent.children.push(/* reference */ );
            }
        }
    }
    
    // Step 3: Determine nodeType
    let has_parent: HashSet<String> = nodes.iter()
        .filter_map(|n| n.parent_id.clone())
        .collect();
    let has_children: HashSet<String> = child_assignments.iter()
        .map(|(p, _)| p.clone())
        .collect();
    
    for node in node_map.values_mut() {
        node.node_type = if !has_parent.contains(&node.base.id) {
            NodeType::Root
        } else if has_children.contains(&node.base.id) {
            NodeType::Container
        } else {
            NodeType::Leaf
        };
    }
    
    // Step 4: Compute depths (BFS from roots)
    let roots: Vec<String> = node_map.values()
        .filter(|n| n.node_type == NodeType::Root)
        .map(|n| n.base.id.clone())
        .collect();
    // BFS to assign depth
    
    // Step 5: Cycle detection
    let cycle_members = detect_cycles(&nodes);
    for id in &cycle_members {
        if let Some(node) = node_map.get_mut(id) {
            node.is_cycle_member = true;
        }
    }
    
    // Step 6: Compute statuses
    compute_statuses(&mut node_map);
    
    // Step 7: Compute progress
    for id in node_map.keys().cloned().collect::<Vec<_>>() {
        let progress = compute_progress(&id, &node_map);
        if let Some(node) = node_map.get_mut(&id) {
            node.progress = progress;
        }
    }
    
    // Step 8: Collect root nodes (roots of the tree)
    // Build final Vec<ComputedNode> with embedded children
    build_recursive_tree(nodes, node_map)
}
```

> **Implementation note**: The recursive tree structure is hard to build with a mutable HashMap due to Rust's borrow rules. The recommended approach is a **two-pass strategy**: first build flat `ComputedNode` vec with status/progress/type computed, then do a recursive tree assembly using `drain` + index maps. Alternatively, use `Rc<RefCell<ComputedNode>>` internally during construction and convert to owned at the end.

---

## 2. `topoSort` (Kahn's Algorithm)

**TypeScript**: `topoSort(nodes: NotionNode[]): NotionNode[]`

```rust
pub fn topo_sort(nodes: &[NotionNode]) -> Vec<NotionNode> {
    let id_to_idx: HashMap<&str, usize> = nodes.iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    
    // In-degree count (number of dependencies per node)
    let mut in_degree: Vec<usize> = vec![0; nodes.len()];
    
    for (i, node) in nodes.iter().enumerate() {
        for dep_id in &node.depends_on_ids {
            if id_to_idx.contains_key(dep_id.as_str()) {
                in_degree[i] += 1;
            }
        }
    }
    
    // Queue: nodes with no remaining dependencies
    let mut queue: VecDeque<usize> = in_degree.iter()
        .enumerate()
        .filter(|(_, &deg)| deg == 0)
        .map(|(i, _)| i)
        .collect();
    
    let mut sorted: Vec<NotionNode> = Vec::with_capacity(nodes.len());
    
    while let Some(idx) = queue.pop_front() {
        sorted.push(nodes[idx].clone());
        
        // Find all nodes that depend on nodes[idx]
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
```

---

## 3. `detectCycles`

```rust
pub fn detect_cycles(nodes: &[NotionNode]) -> HashSet<String> {
    let sorted = topo_sort(nodes);
    let sorted_ids: HashSet<&str> = sorted.iter()
        .map(|n| n.id.as_str())
        .collect();
    
    nodes.iter()
        .filter(|n| !sorted_ids.contains(n.id.as_str()))
        .map(|n| n.id.clone())
        .collect()
}
```

---

## 4. `computeStatuses`

**Complexity note**: Status depends on ancestor statuses and transitive dependencies. Must be computed in topological order.

```rust
pub fn compute_statuses(nodes: &mut HashMap<String, ComputedNode>) {
    // Get topological order of flat nodes
    let flat: Vec<NotionNode> = nodes.values()
        .map(|n| n.base.clone())
        .collect();
    let sorted = topo_sort(&flat);
    
    // Process in topo order so deps are computed before dependents
    for notion_node in &sorted {
        let id = &notion_node.id;
        
        // Check if archived directly
        if notion_node.archived.is_some() {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Archived;
                continue;
            }
        }
        
        // Check if any ancestor is archived (walk parent chain)
        let ancestor_archived = is_ancestor_archived(id, nodes);
        if ancestor_archived {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Archived;
                continue;
            }
        }
        
        // Root nodes are always active
        let node_type = nodes.get(id).map(|n| n.node_type.clone());
        if node_type == Some(NodeType::Root) {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Active;
                continue;
            }
        }
        
        // Check if all dependencies are met (completed)
        let deps_met = notion_node.depends_on_ids.iter().all(|dep_id| {
            nodes.get(dep_id)
                .map(|dep| dep.status == NodeStatus::Completed || dep.status == NodeStatus::Archived)
                .unwrap_or(true)  // unknown dep → assume met
        });
        
        if !deps_met {
            if let Some(n) = nodes.get_mut(id) {
                n.status = NodeStatus::Locked;
                continue;
            }
        }
        
        // Check if completed
        let is_complete = is_node_completed(id, nodes);
        if let Some(n) = nodes.get_mut(id) {
            n.status = if is_complete { NodeStatus::Completed } else { NodeStatus::Active };
        }
    }
}

fn is_node_completed(id: &str, nodes: &HashMap<String, ComputedNode>) -> bool {
    let node = match nodes.get(id) { Some(n) => n, None => return false };
    
    match node.node_type {
        NodeType::Leaf => node.base.done,
        NodeType::Container | NodeType::Root => {
            let children: Vec<&ComputedNode> = nodes.values()
                .filter(|n| n.base.parent_id.as_deref() == Some(id))
                .collect();
            
            let active_children: Vec<&&ComputedNode> = children.iter()
                .filter(|n| n.base.archived.is_none())
                .collect();
            
            if active_children.is_empty() {
                return false;
            }
            
            active_children.iter().all(|n| {
                n.status == NodeStatus::Completed
            })
        }
    }
}

fn is_ancestor_archived(id: &str, nodes: &HashMap<String, ComputedNode>) -> bool {
    let mut current_id = id.to_string();
    loop {
        let parent_id = nodes.get(&current_id)
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
```

---

## 5. `computeProgress`

```rust
pub fn compute_progress(id: &str, nodes: &HashMap<String, ComputedNode>) -> f64 {
    let node = match nodes.get(id) { Some(n) => n, None => return 0.0 };
    
    match node.node_type {
        NodeType::Leaf => if node.base.done { 1.0 } else { 0.0 },
        NodeType::Container | NodeType::Root => {
            let children: Vec<&ComputedNode> = nodes.values()
                .filter(|n| n.base.parent_id.as_deref() == Some(id)
                    && n.base.archived.is_none())
                .collect();
            
            if children.is_empty() {
                return 0.0;
            }
            
            let completed = children.iter()
                .filter(|n| n.status == NodeStatus::Completed)
                .count();
            
            completed as f64 / children.len() as f64
        }
    }
}
```

---

## 6. `computeLayout` (Tree Browser Positioning)

This is the most math-heavy algorithm. Port it faithfully.

```rust
pub struct LayoutOverrides {
    pub positions: HashMap<String, Position>,
}

pub fn compute_layout(
    children: &[ComputedNode],
    overrides: Option<&LayoutOverrides>,
) -> HashMap<String, Position> {
    let h_spacing = 280.0_f64;
    let v_spacing = 120.0_f64;
    let mut positions: HashMap<String, Position> = HashMap::new();
    
    // Step 1: Assign depth columns (x = depth * h_spacing)
    let depths = compute_depths_within_subtree(children);
    
    for (id, depth) in &depths {
        let pos = positions.entry(id.clone()).or_default();
        pos.x = *depth as f64 * h_spacing;
    }
    
    // Step 2: Zone-band allocation (group by parent, assign Y bands)
    assign_y_bands(children, &mut positions, v_spacing);
    
    // Step 3: Centering pass (3 iterations: children → parents)
    for _ in 0..3 {
        center_parents_on_children(children, &mut positions);
    }
    
    // Step 4: Overlap resolution
    resolve_overlaps(&mut positions, v_spacing);
    
    // Step 5: Normalize Y (shift so min Y = 0)
    normalize_y(&mut positions);
    
    // Step 6: Apply manual overrides
    if let Some(ov) = overrides {
        for (id, pos) in &ov.positions {
            if let Some(p) = positions.get_mut(id) {
                *p = pos.clone();
            }
        }
    }
    
    positions
}

fn resolve_overlaps(positions: &mut HashMap<String, Position>, min_spacing: f64) {
    let ids: Vec<String> = positions.keys().cloned().collect();
    let mut changed = true;
    let mut iters = 0;
    
    while changed && iters < 50 {
        changed = false;
        iters += 1;
        
        for i in 0..ids.len() {
            for j in (i+1)..ids.len() {
                let pi = positions[&ids[i]].clone();
                let pj = positions[&ids[j]].clone();
                
                // Only resolve overlaps in same column (same x)
                if (pi.x - pj.x).abs() > 1.0 { continue; }
                
                let dy = (pi.y - pj.y).abs();
                if dy < min_spacing {
                    let push = (min_spacing - dy) / 2.0 + 1.0;
                    let dir = if pi.y <= pj.y { -1.0 } else { 1.0 };
                    
                    positions.get_mut(&ids[i]).unwrap().y += dir * push;
                    positions.get_mut(&ids[j]).unwrap().y -= dir * push;
                    changed = true;
                }
            }
        }
    }
}
```

---

## 7. `computeRadialMapLayout`

```rust
pub fn compute_radial_map_layout(all_nodes: &[ComputedNode]) -> Vec<MapNode> {
    let roots: Vec<&ComputedNode> = all_nodes.iter()
        .filter(|n| n.node_type == NodeType::Root)
        .collect();
    
    if roots.is_empty() {
        return vec![];
    }
    
    let num_roots = roots.len() as f64;
    let sector_angle = std::f64::consts::TAU / num_roots;
    let ring_radius = [0.0_f64, 300.0, 600.0, 900.0, 1200.0]; // pixels per ring
    
    let mut map_nodes: Vec<MapNode> = Vec::new();
    
    for (i, root) in roots.iter().enumerate() {
        let base_angle = i as f64 * sector_angle;
        let sector_center = Position {
            x: ring_radius[1] * base_angle.cos(),
            y: ring_radius[1] * base_angle.sin(),
        };
        let sector_color = root.base.color.as_ref()
            .map(color_to_hex)
            .unwrap_or_else(|| "#7c3aed".to_string());
        
        // Place root at ring 0 (center)
        map_nodes.push(MapNode {
            id: root.base.id.clone(),
            x: 0.0,
            y: 0.0,
            ring: 0,
            sector_id: root.base.id.clone(),
            sector_color: sector_color.clone(),
            sector_angle: base_angle,
            sector_center: sector_center.clone(),
            // ... other fields
        });
        
        // BFS through descendants, placing at increasing rings
        place_descendants_radially(
            root,
            all_nodes,
            1,
            base_angle,
            sector_angle,
            &sector_color,
            &root.base.id,
            &sector_center,
            &ring_radius,
            &mut map_nodes,
        );
    }
    
    map_nodes
}

fn place_descendants_radially(
    parent: &ComputedNode,
    all_nodes: &[ComputedNode],
    ring: usize,
    center_angle: f64,
    available_angle: f64,
    sector_color: &str,
    sector_id: &str,
    sector_center: &Position,
    ring_radius: &[f64],
    output: &mut Vec<MapNode>,
) {
    let children: Vec<&ComputedNode> = all_nodes.iter()
        .filter(|n| n.base.parent_id.as_deref() == Some(&parent.base.id))
        .collect();
    
    if children.is_empty() || ring >= ring_radius.len() {
        return;
    }
    
    let n = children.len() as f64;
    let angle_step = available_angle / n.max(1.0);
    let r = ring_radius[ring];
    
    for (j, child) in children.iter().enumerate() {
        let angle = center_angle - available_angle / 2.0 + angle_step * (j as f64 + 0.5);
        
        output.push(MapNode {
            id: child.base.id.clone(),
            x: r * angle.cos(),
            y: r * angle.sin(),
            ring: ring as u32,
            sector_id: sector_id.to_string(),
            sector_color: sector_color.to_string(),
            sector_angle: angle,
            sector_center: sector_center.clone(),
            // ... other fields from child
        });
        
        place_descendants_radially(
            child,
            all_nodes,
            ring + 1,
            angle,
            angle_step,
            sector_color,
            sector_id,
            sector_center,
            ring_radius,
            output,
        );
    }
}
```

---

## 8. `getCompassSpokes`

```rust
pub struct CompassSpoke<'a> {
    pub root: &'a ComputedNode,
    pub next_node: &'a ComputedNode,
    pub path_to_goal: Vec<&'a ComputedNode>,
    pub last_completed_node: Option<&'a ComputedNode>,
    pub pinned: bool,
}

pub fn get_compass_spokes<'a>(
    all_nodes: &'a [ComputedNode],
) -> (Vec<CompassSpoke<'a>>, Vec<CompassSpoke<'a>>) {
    let roots: Vec<&ComputedNode> = all_nodes.iter()
        .filter(|n| n.node_type == NodeType::Root && n.status == NodeStatus::Active)
        .collect();
    
    let mut pinned: Vec<CompassSpoke> = Vec::new();
    let mut unpinned: Vec<CompassSpoke> = Vec::new();
    
    for root in roots {
        // Find active leaf nodes in this subtree
        let active_leaves: Vec<&ComputedNode> = all_nodes.iter()
            .filter(|n| n.status == NodeStatus::Active
                && n.node_type == NodeType::Leaf
                && is_descendant_of(n, root, all_nodes))
            .collect();
        
        for leaf in active_leaves {
            // Build path from leaf up to nearest boss-level milestone
            let path = build_path_to_milestone(leaf, all_nodes);
            
            let spoke = CompassSpoke {
                root,
                next_node: leaf,
                path_to_goal: path,
                last_completed_node: find_last_completed_in_chain(leaf, all_nodes),
                pinned: leaf.base.pinned || root.base.pinned,
            };
            
            if spoke.pinned {
                pinned.push(spoke);
            } else {
                unpinned.push(spoke);
            }
        }
    }
    
    (pinned, unpinned)
}
```

---

## 9. `getBreadcrumbPath`

```rust
pub fn get_breadcrumb_path<'a>(
    node: &'a ComputedNode,
    all_nodes: &'a [ComputedNode],
) -> Vec<&'a ComputedNode> {
    let node_map: HashMap<&str, &ComputedNode> = all_nodes.iter()
        .map(|n| (n.base.id.as_str(), n))
        .collect();
    
    let mut path: Vec<&ComputedNode> = vec![node];
    let mut current = node;
    
    while let Some(parent_id) = &current.base.parent_id {
        match node_map.get(parent_id.as_str()) {
            Some(parent) => {
                path.push(parent);
                current = parent;
            }
            None => break,
        }
    }
    
    path.reverse();
    path
}
```

---

## Key Porting Notes

1. **HashMap borrow conflicts**: The biggest challenge when porting is that many algorithms need to both read and write to a `HashMap<String, ComputedNode>`. Use a two-phase approach: collect IDs first, then mutate.

2. **Recursive types**: `ComputedNode` containing `Vec<ComputedNode>` is fine in Rust since `Vec` is heap-allocated. No `Box` needed.

3. **Lifetime annotations**: Functions returning references into `all_nodes` slices (like `get_breadcrumb_path`) need explicit lifetime annotations.

4. **Float precision**: All coordinate math uses `f64`. Match TypeScript's `number` type exactly.

5. **Iteration order**: Rust `HashMap` iteration is unordered. Where order matters (e.g., layout rendering), collect to `Vec` and sort first.

6. **Test coverage**: Port the Vitest tests from TypeScript to `#[cfg(test)]` modules. The graph algorithms have clear inputs/outputs, making them ideal for unit tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_topo_sort_simple_chain() {
        let nodes = vec![
            make_node("b", vec!["a"]),
            make_node("a", vec![]),
        ];
        let sorted = topo_sort(&nodes);
        assert_eq!(sorted[0].id, "a");
        assert_eq!(sorted[1].id, "b");
    }
    
    #[test]
    fn test_cycle_detection() {
        let nodes = vec![
            make_node("a", vec!["b"]),
            make_node("b", vec!["a"]),
        ];
        let cycles = detect_cycles(&nodes);
        assert!(cycles.contains("a"));
        assert!(cycles.contains("b"));
    }
}
```
