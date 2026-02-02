//! SkillTreeSource - Stats from allocated skill tree nodes

use crate::source::StatSource;
use crate::stat_block::StatAccumulator;
use crate::types::SkillNodeId;
use loot_core::types::StatType;
use std::collections::HashMap;

/// Stats from skill tree nodes
///
/// This holds a flat list of allocated node IDs.
/// Tree structure, connections, and path validation are handled elsewhere.
/// stat_manager only cares about "what nodes give what stats".
pub struct SkillTreeSource {
    /// List of allocated node IDs
    pub allocated_nodes: Vec<SkillNodeId>,
    /// Mapping from node ID to stat modifiers
    node_stats: HashMap<String, Vec<NodeModifier>>,
}

/// A stat modifier from a skill node
#[derive(Debug, Clone)]
pub struct NodeModifier {
    pub stat: StatType,
    pub value: f64,
    /// Whether this is a "more" multiplier instead of "increased"
    pub is_more: bool,
}

impl SkillTreeSource {
    /// Create a new skill tree source with no allocated nodes
    pub fn new() -> Self {
        SkillTreeSource {
            allocated_nodes: Vec::new(),
            node_stats: HashMap::new(),
        }
    }

    /// Create with pre-defined node stats mapping
    pub fn with_node_stats(node_stats: HashMap<String, Vec<NodeModifier>>) -> Self {
        SkillTreeSource {
            allocated_nodes: Vec::new(),
            node_stats,
        }
    }

    /// Allocate a node
    pub fn allocate(&mut self, node_id: SkillNodeId) {
        if !self.allocated_nodes.contains(&node_id) {
            self.allocated_nodes.push(node_id);
        }
    }

    /// Deallocate a node
    pub fn deallocate(&mut self, node_id: &SkillNodeId) {
        self.allocated_nodes.retain(|n| n != node_id);
    }

    /// Register stats for a node ID
    pub fn register_node(&mut self, node_id: String, modifiers: Vec<NodeModifier>) {
        self.node_stats.insert(node_id, modifiers);
    }

    /// Get the modifiers for a node
    pub fn get_node_modifiers(&self, node_id: &str) -> Option<&Vec<NodeModifier>> {
        self.node_stats.get(node_id)
    }
}

impl Default for SkillTreeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl StatSource for SkillTreeSource {
    fn id(&self) -> &str {
        "skill_tree"
    }

    fn priority(&self) -> i32 {
        100 // Skill tree applies after gear
    }

    fn apply(&self, stats: &mut StatAccumulator) {
        for node_id in &self.allocated_nodes {
            if let Some(modifiers) = self.node_stats.get(&node_id.0) {
                for modifier in modifiers {
                    apply_node_modifier(stats, modifier);
                }
            }
        }
    }
}

fn apply_node_modifier(stats: &mut StatAccumulator, modifier: &NodeModifier) {
    // For "more" multipliers, we need to track them separately
    // For now, apply as "increased" (the StatAccumulator handles the distinction)
    if modifier.is_more {
        // More multipliers need special handling
        match modifier.stat {
            StatType::IncreasedPhysicalDamage => {
                stats.physical_damage_more.push(modifier.value / 100.0);
            }
            StatType::IncreasedLife => {
                stats.life_more.push(modifier.value / 100.0);
            }
            // Add more "more" multiplier cases as needed
            _ => {
                // Default: apply as increased for stats that don't have more tracking
                stats.apply_stat_type(modifier.stat, modifier.value);
            }
        }
    } else {
        stats.apply_stat_type(modifier.stat, modifier.value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tree_allocation() {
        let mut tree = SkillTreeSource::new();
        tree.allocate("node_1".into());
        tree.allocate("node_2".into());

        assert_eq!(tree.allocated_nodes.len(), 2);

        // Allocating same node twice should not duplicate
        tree.allocate("node_1".into());
        assert_eq!(tree.allocated_nodes.len(), 2);
    }

    #[test]
    fn test_skill_tree_deallocation() {
        let mut tree = SkillTreeSource::new();
        tree.allocate("node_1".into());
        tree.allocate("node_2".into());

        tree.deallocate(&"node_1".into());
        assert_eq!(tree.allocated_nodes.len(), 1);
        assert_eq!(tree.allocated_nodes[0].0, "node_2");
    }

    #[test]
    fn test_skill_tree_apply() {
        let mut tree = SkillTreeSource::new();
        tree.register_node(
            "life_node".to_string(),
            vec![NodeModifier {
                stat: StatType::AddedLife,
                value: 20.0,
                is_more: false,
            }],
        );
        tree.allocate("life_node".into());

        let mut acc = StatAccumulator::new();
        tree.apply(&mut acc);

        assert!((acc.life_flat - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_skill_tree_priority() {
        let tree = SkillTreeSource::new();
        assert_eq!(tree.priority(), 100);
    }
}
