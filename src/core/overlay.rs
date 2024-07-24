use i_float::point::IntPoint;
use i_shape::int::path::{IntPath, PointPathExtension};
use i_shape::int::shape::{IntShape, PointsCount};
use crate::fill::fill_segments::FillSegments;

use crate::{split::{shape_edge::ShapeEdge, shape_count::ShapeCount}, fill::{segment::Segment}};
use crate::util::SwapRemoveIndex;
use crate::core::fill_rule::FillRule;
use crate::core::overlay_rule::OverlayRule;
use crate::fill::segment::{CLIP_BOTH, SUBJ_BOTH};
use crate::x_segment::XSegment;
use crate::core::solver::Solver;
use crate::line_range::LineRange;
use crate::split::solver::SplitSolver;
use crate::vector::vector::VectorShape;

use super::overlay_graph::OverlayGraph;

/// Specifies the type of shape being processed, influencing how the shape participates in Boolean operations.
/// Note: All operations except for `Difference` are commutative, meaning the order of `Subject` and `Clip` shapes does not impact the outcome.
/// - `Subject`: The primary shape(s) for operations. Acts as the base layer in the operation.
/// - `Clip`: The modifying shape(s) that are applied to the `Subject`. Determines how the `Subject` is altered or intersected.
#[derive(Debug, Clone, Copy)]
pub enum ShapeType {
    Subject,
    Clip,
}

/// This struct is essential for describing and uploading the geometry or shapes required to construct an `OverlayGraph`. It prepares the necessary data for boolean operations.
#[derive(Clone)]
pub struct Overlay {
    edges: Vec<ShapeEdge>,
}

impl Overlay {
    /// Constructs a new `Overlay` instance, initializing it with a capacity that should closely match the total count of edges from all shapes being processed.
    /// This pre-allocation helps in optimizing memory usage and performance.
    /// - `capacity`: The initial capacity for storing edge data. Ideally, this should be set to the sum of the edges of all shapes to be added to the overlay, ensuring efficient data management.
    pub fn new(capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
        }
    }

    /// Creates a new `Overlay` instance and initializes it with subject and clip paths.
    /// - `subject_paths`: An array of paths that together define the subject shape.
    /// - `clip_paths`: An array of paths that together define the clip shape.
    pub fn with_paths(subject_paths: &[IntPath], clip_paths: &[IntPath]) -> Self {
        let mut overlay = Self::new(subject_paths.points_count() + clip_paths.points_count());
        overlay.add_paths(subject_paths, ShapeType::Subject);
        overlay.add_paths(clip_paths, ShapeType::Clip);
        overlay
    }

    /// Creates a new `Overlay` instance and initializes it with subject and clip shapes.
    /// - `subject_shapes`: An array of shapes to be used as the subject in the overlay operation.
    /// - `clip_shapes`: An array of shapes to be used as the clip in the overlay operation.
    pub fn with_shapes(subject_shapes: &[IntShape], clip_shapes: &[IntShape]) -> Self {
        let mut overlay = Self::new(subject_shapes.points_count() + clip_shapes.points_count());
        overlay.add_shapes(subject_shapes, ShapeType::Subject);
        overlay.add_shapes(clip_shapes, ShapeType::Clip);
        overlay
    }

    /// Adds a single path to the overlay as either subject or clip paths.
    /// - `path`: A reference to a `IntPath` instance to be added.
    /// - `shape_type`: Specifies the role of the added path in the overlay operation, either as `Subject` or `Clip`.
    pub fn add_path(&mut self, path: &[IntPoint], shape_type: ShapeType) {
        if let Some(mut edges) = path.to_vec().removed_degenerates().edges(shape_type) {
            self.edges.append(&mut edges);
        }
    }

    /// Adds multiple paths to the overlay as either subject or clip paths.
    /// - `paths`: An array of `IntPath` instances to be added to the overlay.
    /// - `shape_type`: Specifies the role of the added paths in the overlay operation, either as `Subject` or `Clip`.
    pub fn add_paths(&mut self, paths: &[IntPath], shape_type: ShapeType) {
        for path in paths.iter() {
            self.add_path(path, shape_type);
        }
    }

    /// Adds a single shape to the overlay as either a subject or clip shape.
    /// - `shape`: A reference to a `IntShape` instance to be added.
    /// - `shape_type`: Specifies the role of the added shape in the overlay operation, either as `Subject` or `Clip`.
    pub fn add_shape(&mut self, shape: &IntShape, shape_type: ShapeType) {
        self.add_paths(&shape, shape_type);
    }

    /// Adds multiple shapes to the overlay as either subject or clip shapes.
    /// - `shapes`: An array of `IntShape` instances to be added to the overlay.
    /// - `shape_type`: Specifies the role of the added shapes in the overlay operation, either as `Subject` or `Clip`.
    pub fn add_shapes(&mut self, shapes: &[IntShape], shape_type: ShapeType) {
        for shape in shapes.iter() {
            self.add_paths(&shape, shape_type);
        }
    }

    /// Convert into segments from the added paths or shapes according to the specified fill rule.
    /// - `fill_rule`: The fill rule to use when determining the inside of shapes.
    /// - `solver`: Type of solver to use.
    pub fn into_segments(self, fill_rule: FillRule, solver: Solver) -> Vec<Segment> {
        if self.edges.is_empty() {
            return Vec::new();
        }

        let mut segments = self.prepare_segments(fill_rule, solver);

        segments.filter();

        return segments;
    }

    /// Convert into vector shapes from the added paths or shapes, applying the specified fill and overlay rules. This method is particularly useful for development purposes and for creating visualizations in educational demos, where understanding the impact of different rules on the final geometry is crucial.
    /// - `fill_rule`: The fill rule to use for the shapes.
    /// - `overlay_rule`: The overlay rule to apply.
    /// - `solver`: Type of solver to use.
    pub fn into_vectors(self, fill_rule: FillRule, overlay_rule: OverlayRule, solver: Solver) -> Vec<VectorShape> {
        if self.edges.is_empty() {
            return Vec::new();
        }
        let graph = OverlayGraph::new(self.prepare_segments(fill_rule, solver));
        let vectors = graph.extract_vectors(overlay_rule);

        return vectors;
    }

    /// Convert into `OverlayGraph` from the added paths or shapes using the specified fill rule. This graph is the foundation for executing boolean operations, allowing for the analysis and manipulation of the geometric data. The `OverlayGraph` created by this method represents a preprocessed state of the input shapes, optimized for the application of boolean operations based on the provided fill rule.
    /// - `fill_rule`: Specifies the rule for determining filled areas within the shapes, influencing how the resulting graph represents intersections and unions.
    pub fn into_graph(self, fill_rule: FillRule) -> OverlayGraph {
        OverlayGraph::new(self.into_segments(fill_rule, Solver::AUTO))
    }

    /// Convert into `OverlayGraph` from the added paths or shapes using the specified fill rule. This graph is the foundation for executing boolean operations, allowing for the analysis and manipulation of the geometric data. The `OverlayGraph` created by this method represents a preprocessed state of the input shapes, optimized for the application of boolean operations based on the provided fill rule.
    /// - `fill_rule`: Specifies the rule for determining filled areas within the shapes, influencing how the resulting graph represents intersections and unions.
    /// - `solver`: Type of solver to use.
    pub fn into_graph_with_solver(self, fill_rule: FillRule, solver: Solver) -> OverlayGraph {
        OverlayGraph::new(self.into_segments(fill_rule, solver))
    }

    fn prepare_segments(self, fill_rule: FillRule, solver: Solver) -> Vec<Segment> {
        let mut sorted_list = self.edges;
        sorted_list.sort_by(|a, b| a.x_segment.cmp(&b.x_segment));

        let is_list = SplitSolver { solver, range: sorted_list.y_range() }.split(&mut sorted_list);

        let mut segments = sorted_list.segments();

        segments.fill(fill_rule, is_list);

        segments
    }
}

trait CreateEdges {
    fn edges(&self, shape_type: ShapeType) -> Option<Vec<ShapeEdge>>;
}

impl CreateEdges for IntPath {
    fn edges(&self, shape_type: ShapeType) -> Option<Vec<ShapeEdge>> {
        let n = self.len();
        if n < 3 {
            return None;
        }

        let mut edges = vec![ShapeEdge::ZERO; n];

        let i0 = n - 1;
        let mut p0 = self[i0];

        let mut y_min = p0.y;
        let mut y_max = p0.y;

        for i in 0..n {
            let p1 = self[i];
            y_min = y_min.min(p1.y);
            y_max = y_max.max(p1.y);

            let value = if p0 < p1 { 1 } else { -1 };
            match shape_type {
                ShapeType::Subject => {
                    edges[i] = ShapeEdge::new(p0, p1, ShapeCount::new(value, 0));
                }
                ShapeType::Clip => {
                    edges[i] = ShapeEdge::new(p0, p1, ShapeCount::new(0, value));
                }
            }

            p0 = p1
        }

        Some(edges)
    }
}

trait Filter {
    fn filter(&mut self);
}

impl Filter for Vec<Segment> {
    fn filter(&mut self) {
        let mut modified = false;
        let mut i = 0;
        while i < self.len() {
            let fill = self[i].fill;
            if fill == 0 || fill == SUBJ_BOTH || fill == CLIP_BOTH {
                modified = true;
                self.swap_remove_index(i);
            } else {
                i += 1
            }
        }

        if modified {
            self.sort_by(|a, b| a.seg.cmp(&b.seg));
        }
    }
}

trait ShapeEdgeVecExtension {
    fn segments(self) -> Vec<Segment>;
    fn y_range(&self) -> LineRange;
}

impl ShapeEdgeVecExtension for Vec<ShapeEdge> {
    fn segments(self) -> Vec<Segment> {
        let mut segments = Vec::with_capacity(self.len());

        let mut prev = ShapeEdge { x_segment: XSegment { a: IntPoint::ZERO, b: IntPoint::ZERO }, count: ShapeCount { subj: 0, clip: 0 } };

        for next in self.into_iter() {
            if prev.x_segment == next.x_segment {
                prev.count = prev.count.add(next.count)
            } else {
                if !prev.count.is_empty() {
                    segments.push(Segment::new(&prev));
                }
                prev = next;
            }
        }

        if !prev.count.is_empty() {
            segments.push(Segment::new(&prev));
        }

        return segments;
    }

    fn y_range(&self) -> LineRange {
        let mut min_y = self[0].x_segment.a.y;
        let mut max_y = min_y;

        for edge in self.iter() {
            min_y = min_y.min(edge.x_segment.a.y);
            max_y = max_y.max(edge.x_segment.a.y);
            min_y = min_y.min(edge.x_segment.b.y);
            max_y = max_y.max(edge.x_segment.b.y);
        }

        LineRange { min: min_y, max: max_y }
    }
}
