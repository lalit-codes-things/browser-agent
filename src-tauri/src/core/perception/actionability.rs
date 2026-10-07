// Actionability filter.
//
// C-81: accessibility/rendered-state/geometry analysis must produce
//        semantic actionability filtering and mutation-noise suppression.
//
// This module decides whether a perceived node is actionable for the
// allowed typed action surface.

use crate::core::perception::graph::GraphNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actionability {
    Clickable,
    Typeable,
    Selectable,
    Scrollable,
    NotActionable,
}

pub fn classify(node: &GraphNode) -> Actionability {
    if node.actionable {
        if node.role == "button" || node.role == "link" {
            Actionability::Clickable
        } else if node.role == "textbox" || node.role == "searchbox" {
            Actionability::Typeable
        } else if node.role == "combobox" || node.role == "listbox" {
            Actionability::Selectable
        } else {
            Actionability::NotActionable
        }
    } else {
        Actionability::NotActionable
    }
}
