use i_float::int::point::IntPoint;
use i_shape::int::count::PointsCount;
use i_shape::int::path::IntPath;
use i_shape::int::shape::{IntContour, IntShape};
use crate::core::fill_rule::FillRule;
use crate::core::link::OverlayLinkBuilder;
use crate::core::overlay::ShapeType;
use crate::core::solver::Solver;
use crate::segm::build::BuildSegments;
use crate::segm::segment::{Segment, ToSegment};
use crate::segm::shape_count::ShapeCount;
use crate::string::graph::StringGraph;
use crate::string::line::IntLine;

#[derive(Clone)]
pub struct StringOverlay {
    pub(super) segments: Vec<Segment>,
}

impl StringOverlay {
    /// Constructs a new `StringOverlay` instance, initializing it with a capacity that should closely match the total count of edges from all shapes being processed.
    /// This pre-allocation helps in optimizing memory usage and performance.
    /// - `capacity`: The initial capacity for storing edge data. Ideally, this should be set to the sum of the edges of all shapes to be added to the overlay, ensuring efficient data management.
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
        }
    }

    /// Creates a new `StringOverlay` instance and initializes it with a single shape path.
    /// - `contour`: A path to be used in the overlay operation as a closed shape.
    #[inline]
    pub fn with_shape_contour(contour: &[IntPoint]) -> Self {
        let mut overlay = Self::new(contour.len());
        overlay.add_shape_contour(contour);
        overlay
    }

    /// Creates a new `StringOverlay` instance and initializes it with multiple shape paths.
    /// - `contours`: An array of paths that together define multiple shapes.
    #[inline]
    pub fn with_shape_contours(contours: &[IntContour]) -> Self {
        let mut overlay = Self::new(contours.points_count());
        overlay.add_shape_contours(contours);
        overlay
    }

    /// Creates a new `StringOverlay` instance and initializes it with subject and clip shapes.
    /// - `shape`: A shape to be used in the overlay operation.
    #[inline]
    pub fn with_shape(shape: &[IntContour]) -> Self {
        let mut overlay = Self::new(shape.points_count());
        overlay.add_shape_contours(shape);
        overlay
    }

    /// Creates a new `StringOverlay` instance and initializes it with subject and clip shapes.
    /// - `shapes`: An array of shapes to be used in the overlay operation.
    #[inline]
    pub fn with_shapes(shapes: &[IntShape]) -> Self {
        let mut overlay = Self::new(shapes.points_count());
        overlay.add_shapes(shapes);
        overlay
    }

    /// Adds a path to the overlay using an iterator, allowing for more flexible path input.
    /// This function is particularly useful when working with dynamically generated paths or
    /// when paths are not directly stored in a collection.
    /// - `iter`: An iterator over references to `IntPoint` that defines the path.
    #[inline]
    pub fn add_shape_contour_iter<I: Iterator<Item = IntPoint>>(&mut self, iter: I) {
        self.segments.append_path_iter(iter, ShapeType::Subject);
    }

    /// Adds a single path to the overlay as a shape paths.
    /// - `contour`: An array of points that form a closed path.
    #[inline]
    pub fn add_shape_contour(&mut self, contour: &[IntPoint]) {
        self.add_shape_contour_iter(contour.iter().copied());
    }

    /// Adds multiple paths to the overlay as shape paths.
    /// - `contours`: An array of `IntContour` instances to be added to the overlay.
    pub fn add_shape_contours(&mut self, contours: &[IntContour]) {
        for contour in contours.iter() {
            self.add_shape_contour(contour);
        }
    }

    /// Adds a list of shape to the overlay.
    /// - `shapes`: An array of `IntShape` instances to be added to the overlay.
    #[inline]
    pub fn add_shapes(&mut self, shapes: &[IntShape]) {
        for shape in shapes {
            self.add_shape_contours(shape);
        }
    }

    /// Adds a single line (open path) to the overlay.
    /// - `line`: An `IntLine` representing the open line (defined by two points).
    #[inline]
    pub fn add_string_line(&mut self, line: IntLine) {
        if line[0] != line[1] {
            self.segments.push(line.to_segment(ShapeCount::new(0, 1)));
        }
    }

    /// Adds multiple lines (open paths) to the overlay.
    /// - `lines`: An array of `IntLine` instances to be added.
    #[inline]
    pub fn add_string_lines(&mut self, lines: &[IntLine]) {
        for &line in lines {
            self.add_string_line(line);
        }
    }

    /// Adds a path (a sequence of points) as an open or closed string to the overlay.
    /// - `path`: A reference to an array of `IntPoint` representing the path.
    /// - `is_open`: A boolean flag indicating whether the path is open (true) or closed (false).
    #[inline]
    pub fn add_string_path(&mut self, path: &[IntPoint], is_open: bool) {
        let mut a = if let Some(&p) = path.first() { p } else { return; };
        for &b in path.iter().skip(1) {
            self.add_string_line([a, b]);
            a = b;
        }

        if !is_open && path.len() > 2 {
            let &a = path.first().unwrap();
            let &b = path.last().unwrap();
            self.add_string_line([b, a])
        }
    }

    /// Adds multiple paths as open or closed strings to the overlay.
    /// - `paths`: An array of `IntPath` instances (each a sequence of points).
    /// - `is_open`: A boolean flag indicating whether the paths are open (true) or closed (false).
    #[inline]
    pub fn add_string_paths(&mut self, paths: &[IntPath], is_open: bool) {
        for path in paths {
            self.add_string_path(path, is_open);
        }
    }

    /// Converts the overlay into a `StringGraph`, using the specified `FillRule`.
    /// This graph is used for string operations, enabling analysis and manipulation of geometric data.
    /// - `fill_rule`: The rule that defines how to fill shapes (e.g., non-zero, even-odd).
    #[inline]
    pub fn into_graph(self, fill_rule: FillRule) -> StringGraph {
        self.into_graph_with_solver(fill_rule, Default::default())
    }

    /// Converts the overlay into a `StringGraph`, with an additional option to specify a custom solver.
    /// This graph is used for string operations, enabling analysis and manipulation of geometric data.
    /// - `fill_rule`: The rule that defines how to fill shapes (e.g., non-zero, even-odd).
    /// - `solver`: A solver type to be used for advanced control over the graph building process.
    #[inline]
    pub fn into_graph_with_solver(self, fill_rule: FillRule, solver: Solver) -> StringGraph {
        let links = OverlayLinkBuilder::build_string(self.segments, fill_rule, solver);
        StringGraph::new(solver, links)
    }
}