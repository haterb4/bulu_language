//! Control flow analysis for IR
//!
//! This module provides functionality for analyzing control flow in IR code,
//! including building control flow graphs and performing various analyses.

use super::ir::*;
use crate::error::Result;
use std::collections::{HashMap, HashSet, VecDeque};

/// Control flow analyzer
pub struct ControlFlowAnalyzer;

impl ControlFlowAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Build control flow graph for a function
    pub fn build_cfg(&self, function: &IrFunction) -> Result<ControlFlowGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut label_to_node: HashMap<String, usize> = HashMap::new();
        
        // Create nodes for each basic block
        for (i, block) in function.basic_blocks.iter().enumerate() {
            label_to_node.insert(block.label.clone(), i);
            nodes.push(CfgNode {
                id: i,
                block_label: block.label.clone(),
                predecessors: Vec::new(),
                successors: Vec::new(),
            });
        }
        
        // Create edges based on terminators
        for (i, block) in function.basic_blocks.iter().enumerate() {
            match &block.terminator {
                IrTerminator::Branch(target_label) => {
                    if let Some(&target_id) = label_to_node.get(target_label) {
                        edges.push(CfgEdge {
                            from: i,
                            to: target_id,
                            condition: None,
                        });
                        nodes[i].successors.push(target_id);
                        nodes[target_id].predecessors.push(i);
                    }
                }
                
                IrTerminator::ConditionalBranch { condition, true_label, false_label } => {
                    // True branch
                    if let Some(&true_id) = label_to_node.get(true_label) {
                        edges.push(CfgEdge {
                            from: i,
                            to: true_id,
                            condition: Some(condition.clone()),
                        });
                        nodes[i].successors.push(true_id);
                        nodes[true_id].predecessors.push(i);
                    }
                    
                    // False branch
                    if let Some(&false_id) = label_to_node.get(false_label) {
                        edges.push(CfgEdge {
                            from: i,
                            to: false_id,
                            condition: None, // Implicit negation of condition
                        });
                        nodes[i].successors.push(false_id);
                        nodes[false_id].predecessors.push(i);
                    }
                }
                
                IrTerminator::Switch { cases, default_label, .. } => {
                    // Case branches
                    for (case_value, case_label) in cases {
                        if let Some(&case_id) = label_to_node.get(case_label) {
                            edges.push(CfgEdge {
                                from: i,
                                to: case_id,
                                condition: Some(case_value.clone()),
                            });
                            nodes[i].successors.push(case_id);
                            nodes[case_id].predecessors.push(i);
                        }
                    }
                    
                    // Default branch
                    if let Some(default_label) = default_label {
                        if let Some(&default_id) = label_to_node.get(default_label) {
                            edges.push(CfgEdge {
                                from: i,
                                to: default_id,
                                condition: None,
                            });
                            nodes[i].successors.push(default_id);
                            nodes[default_id].predecessors.push(i);
                        }
                    }
                }
                
                IrTerminator::Return(_) | IrTerminator::Unreachable => {
                    // No successors for return/unreachable
                }
            }
        }
        
        Ok(ControlFlowGraph { nodes, edges })
    }
    
    /// Perform dominance analysis
    /// Returns a map from node ID to its immediate dominator
    pub fn compute_dominators(&self, cfg: &ControlFlowGraph) -> HashMap<usize, Option<usize>> {
        let mut dominators: HashMap<usize, Option<usize>> = HashMap::new();
        
        if cfg.nodes.is_empty() {
            return dominators;
        }
        
        // Entry node dominates itself
        dominators.insert(0, None);
        
        // Initialize all other nodes to be dominated by all nodes
        for i in 1..cfg.nodes.len() {
            dominators.insert(i, None);
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for i in 1..cfg.nodes.len() {
                let node = &cfg.nodes[i];
                
                // Find intersection of dominators of all predecessors
                let mut new_dominator = None;
                
                for &pred_id in &node.predecessors {
                    if let Some(pred_dom) = dominators.get(&pred_id) {
                        if pred_dom.is_some() || pred_id == 0 {
                            if new_dominator.is_none() {
                                new_dominator = Some(pred_id);
                            } else {
                                new_dominator = Some(self.intersect_dominators(
                                    new_dominator.unwrap(),
                                    pred_id,
                                    &dominators,
                                ));
                            }
                        }
                    }
                }
                
                if dominators.get(&i) != Some(&new_dominator) {
                    dominators.insert(i, new_dominator);
                    changed = true;
                }
            }
        }
        
        dominators
    }
    
    /// Find intersection of two dominators
    fn intersect_dominators(&self, mut finger1: usize, mut finger2: usize, dominators: &HashMap<usize, Option<usize>>) -> usize {
        while finger1 != finger2 {
            while finger1 > finger2 {
                if let Some(Some(dom)) = dominators.get(&finger1) {
                    finger1 = *dom;
                } else {
                    break;
                }
            }
            while finger2 > finger1 {
                if let Some(Some(dom)) = dominators.get(&finger2) {
                    finger2 = *dom;
                } else {
                    break;
                }
            }
        }
        finger1
    }
    
    /// Compute post-dominators
    /// Returns a map from node ID to its immediate post-dominator
    pub fn compute_post_dominators(&self, cfg: &ControlFlowGraph) -> HashMap<usize, Option<usize>> {
        let mut post_dominators: HashMap<usize, Option<usize>> = HashMap::new();
        
        if cfg.nodes.is_empty() {
            return post_dominators;
        }
        
        // Find exit nodes (nodes with no successors)
        let mut exit_nodes = Vec::new();
        for (i, node) in cfg.nodes.iter().enumerate() {
            if node.successors.is_empty() {
                exit_nodes.push(i);
            }
        }
        
        // If there's only one exit node, it post-dominates itself
        if exit_nodes.len() == 1 {
            post_dominators.insert(exit_nodes[0], None);
        }
        
        // Initialize all other nodes
        for i in 0..cfg.nodes.len() {
            if !exit_nodes.contains(&i) {
                post_dominators.insert(i, None);
            }
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            // Process nodes in reverse post-order
            for i in (0..cfg.nodes.len()).rev() {
                if exit_nodes.contains(&i) {
                    continue;
                }
                
                let node = &cfg.nodes[i];
                let mut new_post_dominator = None;
                
                for &succ_id in &node.successors {
                    if let Some(succ_post_dom) = post_dominators.get(&succ_id) {
                        if succ_post_dom.is_some() || exit_nodes.contains(&succ_id) {
                            if new_post_dominator.is_none() {
                                new_post_dominator = Some(succ_id);
                            } else {
                                new_post_dominator = Some(self.intersect_post_dominators(
                                    new_post_dominator.unwrap(),
                                    succ_id,
                                    &post_dominators,
                                    &exit_nodes,
                                ));
                            }
                        }
                    }
                }
                
                if post_dominators.get(&i) != Some(&new_post_dominator) {
                    post_dominators.insert(i, new_post_dominator);
                    changed = true;
                }
            }
        }
        
        post_dominators
    }
    
    /// Find intersection of two post-dominators
    fn intersect_post_dominators(
        &self,
        mut finger1: usize,
        mut finger2: usize,
        post_dominators: &HashMap<usize, Option<usize>>,
        exit_nodes: &[usize],
    ) -> usize {
        while finger1 != finger2 {
            while finger1 < finger2 {
                if let Some(Some(post_dom)) = post_dominators.get(&finger1) {
                    finger1 = *post_dom;
                } else if exit_nodes.contains(&finger1) {
                    break;
                } else {
                    break;
                }
            }
            while finger2 < finger1 {
                if let Some(Some(post_dom)) = post_dominators.get(&finger2) {
                    finger2 = *post_dom;
                } else if exit_nodes.contains(&finger2) {
                    break;
                } else {
                    break;
                }
            }
        }
        finger1
    }
    
    /// Detect natural loops in the control flow graph
    pub fn find_natural_loops(&self, cfg: &ControlFlowGraph) -> Vec<NaturalLoop> {
        let mut loops = Vec::new();
        let dominators = self.compute_dominators(cfg);
        
        // Find back edges (edges where target dominates source)
        for edge in &cfg.edges {
            if let (Some(Some(target_dom)), Some(Some(source_dom))) = 
                (dominators.get(&edge.to), dominators.get(&edge.from)) {
                if self.dominates(edge.to, edge.from, &dominators) {
                    // This is a back edge, find the natural loop
                    let loop_nodes = self.find_loop_nodes(edge.from, edge.to, cfg);
                    loops.push(NaturalLoop {
                        header: edge.to,
                        back_edge_source: edge.from,
                        nodes: loop_nodes,
                    });
                }
            }
        }
        
        loops
    }
    
    /// Check if node a dominates node b
    fn dominates(&self, a: usize, b: usize, dominators: &HashMap<usize, Option<usize>>) -> bool {
        if a == b {
            return true;
        }
        
        let mut current = b;
        while let Some(Some(dom)) = dominators.get(&current) {
            if *dom == a {
                return true;
            }
            current = *dom;
        }
        
        false
    }
    
    /// Find all nodes in a natural loop
    fn find_loop_nodes(&self, back_edge_source: usize, header: usize, cfg: &ControlFlowGraph) -> HashSet<usize> {
        let mut loop_nodes = HashSet::new();
        let mut worklist = VecDeque::new();
        
        loop_nodes.insert(header);
        loop_nodes.insert(back_edge_source);
        worklist.push_back(back_edge_source);
        
        while let Some(node_id) = worklist.pop_front() {
            let node = &cfg.nodes[node_id];
            
            for &pred_id in &node.predecessors {
                if !loop_nodes.contains(&pred_id) {
                    loop_nodes.insert(pred_id);
                    worklist.push_back(pred_id);
                }
            }
        }
        
        loop_nodes
    }
    
    /// Perform liveness analysis
    /// Returns sets of live variables at each program point
    pub fn compute_liveness(&self, function: &IrFunction) -> LivenessInfo {
        let cfg = self.build_cfg(function).unwrap();
        let mut live_in: HashMap<usize, HashSet<IrRegister>> = HashMap::new();
        let mut live_out: HashMap<usize, HashSet<IrRegister>> = HashMap::new();
        
        // Initialize
        for i in 0..cfg.nodes.len() {
            live_in.insert(i, HashSet::new());
            live_out.insert(i, HashSet::new());
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            // Process nodes in reverse order
            for i in (0..cfg.nodes.len()).rev() {
                let block = &function.basic_blocks[i];
                
                // Compute live_out[i] = union of live_in[s] for all successors s
                let mut new_live_out = HashSet::new();
                for &succ_id in &cfg.nodes[i].successors {
                    if let Some(succ_live_in) = live_in.get(&succ_id) {
                        new_live_out.extend(succ_live_in.iter().cloned());
                    }
                }
                
                // Compute live_in[i] = use[i] ∪ (live_out[i] - def[i])
                let (use_set, def_set) = self.compute_use_def_sets(block);
                let mut new_live_in = use_set;
                for reg in &new_live_out {
                    if !def_set.contains(reg) {
                        new_live_in.insert(*reg);
                    }
                }
                
                // Check for changes
                if live_in.get(&i) != Some(&new_live_in) || live_out.get(&i) != Some(&new_live_out) {
                    changed = true;
                    live_in.insert(i, new_live_in);
                    live_out.insert(i, new_live_out);
                }
            }
        }
        
        LivenessInfo { live_in, live_out }
    }
    
    /// Compute use and def sets for a basic block
    fn compute_use_def_sets(&self, block: &IrBasicBlock) -> (HashSet<IrRegister>, HashSet<IrRegister>) {
        let mut use_set = HashSet::new();
        let mut def_set = HashSet::new();
        
        // Process instructions in order
        for instruction in &block.instructions {
            // Add uses (operands that are registers and not already defined)
            for operand in &instruction.operands {
                if let IrValue::Register(reg) = operand {
                    if !def_set.contains(reg) {
                        use_set.insert(*reg);
                    }
                }
            }
            
            // Add definition
            if let Some(result_reg) = instruction.result {
                def_set.insert(result_reg);
            }
        }
        
        // Process terminator
        match &block.terminator {
            IrTerminator::Return(Some(value)) => {
                if let IrValue::Register(reg) = value {
                    if !def_set.contains(reg) {
                        use_set.insert(*reg);
                    }
                }
            }
            IrTerminator::ConditionalBranch { condition, .. } => {
                if let IrValue::Register(reg) = condition {
                    if !def_set.contains(reg) {
                        use_set.insert(*reg);
                    }
                }
            }
            IrTerminator::Switch { value, cases, .. } => {
                if let IrValue::Register(reg) = value {
                    if !def_set.contains(reg) {
                        use_set.insert(*reg);
                    }
                }
                for (case_value, _) in cases {
                    if let IrValue::Register(reg) = case_value {
                        if !def_set.contains(reg) {
                            use_set.insert(*reg);
                        }
                    }
                }
            }
            _ => {}
        }
        
        (use_set, def_set)
    }
    
    /// Detect unreachable code
    pub fn find_unreachable_blocks(&self, cfg: &ControlFlowGraph) -> HashSet<usize> {
        let mut reachable = HashSet::new();
        let mut worklist = VecDeque::new();
        
        if !cfg.nodes.is_empty() {
            // Start from entry node (node 0)
            reachable.insert(0);
            worklist.push_back(0);
        }
        
        while let Some(node_id) = worklist.pop_front() {
            let node = &cfg.nodes[node_id];
            
            for &succ_id in &node.successors {
                if !reachable.contains(&succ_id) {
                    reachable.insert(succ_id);
                    worklist.push_back(succ_id);
                }
            }
        }
        
        // Return unreachable nodes
        let mut unreachable = HashSet::new();
        for i in 0..cfg.nodes.len() {
            if !reachable.contains(&i) {
                unreachable.insert(i);
            }
        }
        
        unreachable
    }
}

/// Natural loop information
#[derive(Debug, Clone)]
pub struct NaturalLoop {
    pub header: usize,
    pub back_edge_source: usize,
    pub nodes: HashSet<usize>,
}

/// Liveness analysis results
#[derive(Debug, Clone)]
pub struct LivenessInfo {
    pub live_in: HashMap<usize, HashSet<IrRegister>>,
    pub live_out: HashMap<usize, HashSet<IrRegister>>,
}

/// Reaching definitions analysis
pub struct ReachingDefinitions {
    pub definitions: HashMap<usize, HashSet<Definition>>,
}

/// A definition of a register
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub register: IrRegister,
    pub block_id: usize,
    pub instruction_id: usize,
}

impl ControlFlowAnalyzer {
    /// Compute reaching definitions
    pub fn compute_reaching_definitions(&self, function: &IrFunction) -> ReachingDefinitions {
        let cfg = self.build_cfg(function).unwrap();
        let mut reaching_defs: HashMap<usize, HashSet<Definition>> = HashMap::new();
        
        // Initialize
        for i in 0..cfg.nodes.len() {
            reaching_defs.insert(i, HashSet::new());
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for i in 0..cfg.nodes.len() {
                let block = &function.basic_blocks[i];
                let node = &cfg.nodes[i];
                
                // Compute incoming definitions from predecessors
                let mut incoming_defs = HashSet::new();
                for &pred_id in &node.predecessors {
                    if let Some(pred_defs) = reaching_defs.get(&pred_id) {
                        incoming_defs.extend(pred_defs.iter().cloned());
                    }
                }
                
                // Apply definitions in this block
                let mut block_defs = incoming_defs.clone();
                for (inst_id, instruction) in block.instructions.iter().enumerate() {
                    if let Some(result_reg) = instruction.result {
                        // Remove previous definitions of this register
                        block_defs.retain(|def| def.register != result_reg);
                        
                        // Add new definition
                        block_defs.insert(Definition {
                            register: result_reg,
                            block_id: i,
                            instruction_id: inst_id,
                        });
                    }
                }
                
                // Check for changes
                if reaching_defs.get(&i) != Some(&block_defs) {
                    changed = true;
                    reaching_defs.insert(i, block_defs);
                }
            }
        }
        
        ReachingDefinitions {
            definitions: reaching_defs,
        }
    }
}