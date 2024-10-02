use i_float::point::IntPoint;
use i_shape::int::path::{IntPath, PointPathExtension};
use i_shape::int::shape::{IntShape, IntShapes};
use i_shape::int::simple::Simple;
use crate::bind::hole_point::HolePoint;
use crate::bind::segment::IdSegments;
use crate::bind::solver::ShapeBinder;
use crate::id_point::IdPoint;
use crate::core::overlay_graph::OverlayGraph;
use crate::core::overlay_link::OverlayLink;
use crate::core::overlay_node::OverlayNode;
use crate::core::solver::Solver;
use crate::sort::SmartBinSort;

use super::overlay_rule::OverlayRule;
use super::filter::Filter;

impl OverlayGraph {
    /// Extracts shapes from the overlay graph based on the specified overlay rule. This method is used to retrieve the final geometric shapes after boolean operations have been applied. It's suitable for most use cases where the minimum area of shapes is not a concern.
    /// - `overlay_rule`: The boolean operation rule to apply when extracting shapes from the graph, such as union or intersection.
    /// - Returns: A vector of `IntShape`, representing the geometric result of the applied overlay rule.
    /// # Shape Representation
    /// The output is a `Vec<Vec<Vec<IntPoint>>>`, where:
    /// - The outer `Vec<Shape>` represents a set of shapes.
    /// - Each shape `Vec<Path>` represents a collection of paths, where the first path is the outer boundary, and all subsequent paths are holes in this boundary.
    /// - Each path `Vec<IntPoint>` is a sequence of points, forming a closed path.
    ///
    /// Note: Outer boundary paths have a clockwise order, and holes have a counterclockwise order.
    #[inline(always)]
    pub fn extract_shapes(&self, overlay_rule: OverlayRule) -> IntShapes {
        self.extract_shapes_min_area(overlay_rule, 0)
    }

    /// Extracts shapes from the overlay graph similar to `extract_shapes`, but with an additional constraint on the minimum area of the shapes. This is useful for filtering out shapes that do not meet a certain size threshold, which can be beneficial for eliminating artifacts or noise from the output.
    /// - `overlay_rule`: The boolean operation rule to apply, determining how shapes are combined or subtracted.
    /// - `min_area`: The minimum area threshold for shapes to be included in the result. Shapes with an area smaller than this value will be excluded.
    /// - Returns: A vector of `IntShape` that meet the specified area criteria, representing the cleaned-up geometric result.
    /// # Shape Representation
    /// The output is a `Vec<Vec<Vec<IntPoint>>>`, where:
    /// - The outer `Vec<Shape>` represents a set of shapes.
    /// - Each shape `Vec<Path>` represents a collection of paths, where the first path is the outer boundary, and all subsequent paths are holes in this boundary.
    /// - Each path `Vec<IntPoint>` is a sequence of points, forming a closed path.
    ///
    /// Note: Outer boundary paths have a clockwise order, and holes have a counterclockwise order.
    #[inline]
    pub fn extract_shapes_min_area(&self, overlay_rule: OverlayRule, min_area: i64) -> IntShapes {
        let mut binding = self.links.filter(overlay_rule);
        let visited = binding.as_mut_slice();
        let mut shapes = Vec::new();
        let mut holes = Vec::new();

        let mut link_index = 0;
        while link_index < visited.len() {
            let &count_to_visit = unsafe { visited.get_unchecked(link_index) };
            if count_to_visit == 0 {
                link_index += 1;
                continue;
            }

            let left_top_link = self.find_left_top_link(link_index, visited);
            let link = self.link(left_top_link);
            let is_hole = overlay_rule.is_fill_top(link.fill);

            let start_data = StartPathData::new(is_hole, link, left_top_link);

            let mut path = self.get_path(&start_data, visited);

            if path.validate(min_area) {
                if is_hole {
                    holes.push(path);
                } else {
                    shapes.push(vec![path]);
                }
            }
        }

        shapes.join_holes(&self.solver, holes);

        shapes
    }

    #[inline]
    pub(crate) fn get_path(&self, start_data: &StartPathData, visited: &mut [u8]) -> IntPath {
        let mut link_id = start_data.link_id;
        let mut node_id = start_data.node_id;
        let last_node_id = start_data.last_node_id;

        unsafe {
            *visited.get_unchecked_mut(link_id) -= 1;
        };

        let mut path = IntPath::new();
        path.push(start_data.begin);

        // Find a closed tour
        while node_id != last_node_id {
            let node = self.node(node_id);
            link_id = match node {
                OverlayNode::Bridge(bridge) => {
                    if bridge[0] == link_id { bridge[1] } else { bridge[0] }
                }
                OverlayNode::Cross(indices) => {
                    self.find_nearest_counter_wise_link_to(link_id, node_id, indices, visited)
                }
            };

            let link = self.link(link_id);
            node_id = if link.a.id == node_id {
                path.push(link.a.point);
                link.b.id
            } else {
                path.push(link.b.point);
                link.a.id
            };

            unsafe {
                *visited.get_unchecked_mut(link_id) -= 1;
            };
        }

        path
    }
}

pub(crate) struct StartPathData {
    pub(crate) begin: IntPoint,
    pub(crate) node_id: usize,
    pub(crate) link_id: usize,
    pub(crate) last_node_id: usize,
}

impl StartPathData {
    #[inline(always)]
    pub(crate) fn new(is_hole: bool, link: &OverlayLink, link_id: usize) -> Self {
        if is_hole {
            Self {
                begin: link.b.point,
                node_id: link.a.id,
                link_id,
                last_node_id: link.b.id,
            }
        } else {
            Self {
                begin: link.a.point,
                node_id: link.b.id,
                link_id,
                last_node_id: link.a.id,
            }
        }
    }
}

trait JoinSortedHoles {
    fn join_holes(&mut self, solver: &Solver, holes: Vec<IntPath>);
}

impl JoinSortedHoles for Vec<IntShape> {
    #[inline]
    fn join_holes(&mut self, solver: &Solver, holes: Vec<IntPath>) {
        if self.is_empty() || holes.is_empty() {
            return;
        }

        if self.len() == 1 {
            self[0].reserve_exact(holes.len());
            let mut hole_paths = holes;
            self[0].append(&mut hole_paths);
        } else {
            // Mark: we take first point in the path, that why we get sorted array
            let hole_points: Vec<_> = holes.iter().enumerate()
                .map(|(i, path)| IdPoint::new(i, *path.first().unwrap()))
                .collect();
            self.join_holes_by_points(solver, holes, hole_points);
        }
    }
}

pub(crate) trait JoinHoles {
    fn join_holes_by_points<P: HolePoint>(&mut self, solver: &Solver, holes: Vec<IntPath>, hole_points: Vec<P>);
    fn scan_join<P: HolePoint>(&mut self, solver: &Solver, holes: Vec<IntPath>, hole_points: Vec<P>);
}

impl JoinHoles for Vec<IntShape> {

    #[inline]
    fn join_holes_by_points<P: HolePoint>(&mut self, solver: &Solver, holes: Vec<IntPath>, hole_points: Vec<P>) {
        debug_assert!(self.len() > 1);
        debug_assert!(!hole_points.is_empty());
        self.scan_join(solver, holes, hole_points);
    }

    fn scan_join<P: HolePoint>(&mut self, solver: &Solver, holes: Vec<IntPath>, hole_points: Vec<P>) {
        let x_min = hole_points[0].point().x;
        let x_max = hole_points[hole_points.len() - 1].point().x;

        let capacity = self.iter().fold(0, |s, it| s + it[0].len()) / 2;
        let mut segments = Vec::with_capacity(capacity);
        for (i, shape) in self.iter().enumerate() {
            shape[0].append_id_segments(&mut segments, i, x_min, x_max);
        }

        segments.smart_bin_sort_by(solver, |a, b| a.x_segment.a.x.cmp(&b.x_segment.a.x));

        let solution = ShapeBinder::bind(self.len(), hole_points, segments);

        for (shape_index, &capacity) in solution.children_count_for_parent.iter().enumerate() {
            self[shape_index].reserve_exact(capacity);
        }

        for (hole_index, hole) in holes.into_iter().enumerate() {
            let shape_index = solution.parent_for_child[hole_index];
            self[shape_index].push(hole);
        }
    }
}

pub(crate) trait Validate {
    fn validate(&mut self, min_area: i64) -> bool;
}

impl Validate for IntPath {
    #[inline]
    fn validate(&mut self, min_area: i64) -> bool {
        let slice = self.as_slice();
        if !slice.is_simple() {
            let simple = slice.to_simple();
            let _ = std::mem::replace(self, simple);
        }

        if self.len() < 3 {
            return false;
        }

        if min_area == 0 {
            return true;
        }

        let area = self.unsafe_area();
        let abs_area = area.abs() >> 1;

        abs_area < min_area
    }
}