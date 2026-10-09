// Hit-test validation.
//
// C-155: Phase-1 slice performs basic click hit-test validation before
//        execution; Phase 3 completes recursive OOPIF descent per C-87.

use crate::core::perception::graph::{GeometryBounds, GraphNode};

pub struct HitTest;

impl HitTest {
    pub fn point_in_bounds(point: (i32, i32), bounds: &GeometryBounds) -> bool {
        let GeometryBounds {
            x,
            y,
            width,
            height,
        } = bounds;
        point.0 >= *x
            && point.0 < *x + *width as i32
            && point.1 >= *y
            && point.1 < *y + *height as i32
    }

    pub fn validate_click_target(
        node: &GraphNode,
        _normalized_point: (i32, i32),
    ) -> Result<(), crate::Error> {
        if !node.actionable {
            return Err(crate::Error::Unsupported("target is not actionable".into()));
        }
        if node.bounds.is_none() {
            return Err(crate::Error::InvalidParameter(
                "target has no geometry".into(),
            ));
        }
        Ok(())
    }
}
