// Geometry and visibility analysis.
//
// C-81: accessibility/rendered-state/geometry analysis -> semantic
//        actionability filtering. Nodes without geometry cannot be
//        hit-tested and must not be silently treated as clickable.
//
// Viewport dimensions are runtime-provided, not invented here.

use crate::core::perception::graph::GeometryBounds;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Clipped,
    Offscreen,
    ZeroArea,
}

impl Visibility {
    /// Only Visible nodes are candidates for direct action. Clipped and
    /// Offscreen require scrolling; ZeroArea is not actionable at all.
    pub fn is_actionable_candidate(self) -> bool {
        matches!(self, Self::Visible)
    }
}

pub fn classify_visibility(node: &GeometryBounds, viewport: Viewport) -> Visibility {
    if node.width == 0 || node.height == 0 {
        return Visibility::ZeroArea;
    }
    let fully_outside = node.x >= viewport.width as i32
        || node.y >= viewport.height as i32
        || (node.x + node.width as i32) <= 0
        || (node.y + node.height as i32) <= 0;
    if fully_outside {
        return Visibility::Offscreen;
    }
    let right = node.x + node.width as i32;
    let bottom = node.y + node.height as i32;
    if right > viewport.width as i32 || bottom > viewport.height as i32 || node.x < 0 || node.y < 0 {
        return Visibility::Clipped;
    }
    Visibility::Visible
}

#[cfg(test)]
mod tests {
    use super::*;

    const VP: Viewport = Viewport { width: 1200, height: 800 };

    fn bounds(x: i32, y: i32, w: u32, h: u32) -> GeometryBounds {
        GeometryBounds { x, y, width: w, height: h }
    }

    #[test]
    fn visible_node() {
        assert_eq!(classify_visibility(&bounds(10, 10, 100, 40), VP), Visibility::Visible);
    }

    #[test]
    fn zero_area_is_not_actionable() {
        let v = classify_visibility(&bounds(10, 10, 0, 40), VP);
        assert_eq!(v, Visibility::ZeroArea);
        assert!(!v.is_actionable_candidate());
    }

    #[test]
    fn offscreen_right() {
        assert_eq!(classify_visibility(&bounds(1300, 10, 100, 40), VP), Visibility::Offscreen);
    }

    #[test]
    fn offscreen_above() {
        assert_eq!(classify_visibility(&bounds(10, -100, 100, 40), VP), Visibility::Offscreen);
    }

    #[test]
    fn clipped_partially_visible() {
        assert_eq!(classify_visibility(&bounds(1150, 10, 100, 40), VP), Visibility::Clipped);
        let v = classify_visibility(&bounds(1150, 10, 100, 40), VP);
        assert!(!v.is_actionable_candidate());
    }
}
