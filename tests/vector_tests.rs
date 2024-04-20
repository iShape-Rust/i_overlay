#[cfg(test)]
mod tests {
    use i_float::point::IntPoint;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::{Overlay, ShapeType};
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::core::solver::Solver;
    use i_overlay::vector::vector::VectorEdge;

    #[test]
    fn test_0() {
        let subj = [
            IntPoint::new(-10240, -10240),
            IntPoint::new(-10240, 10240),
            IntPoint::new(10240, 10240),
            IntPoint::new(10240, -10240)
        ];

        let clip = [
            IntPoint::new(-5120, -5120),
            IntPoint::new(-5120, 5120),
            IntPoint::new(5120, 5120),
            IntPoint::new(5120, -5120)
        ];

        let mut overlay = Overlay::new(2);
        overlay.add_path(&subj, ShapeType::Subject);
        overlay.add_path(&clip, ShapeType::Clip);

        let shapes = overlay.build_vectors(FillRule::NonZero, OverlayRule::Subject, Solver::Auto);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].len(), 1);

        let vectors = &shapes[0][0];
        let template = [
            VectorEdge {
                a: IntPoint::new(-10240,-10240),
                b: IntPoint::new(-10240,10240),
                fill: 2
            },
            VectorEdge {
                a: IntPoint::new(-10240,10240),
                b: IntPoint::new(10240,10240),
                fill: 2
            },
            VectorEdge {
                a: IntPoint::new(10240,10240),
                b: IntPoint::new(10240,-10240),
                fill: 2
            },
            VectorEdge {
                a: IntPoint::new(10240,-10240),
                b: IntPoint::new(-10240,-10240),
                fill: 2
            }
        ];

        assert_eq!(vectors.as_slice(), template.as_slice());
    }

    #[test]
    fn test_1() {
        let subj = [
            IntPoint::new(-10240, -10240),
            IntPoint::new(-10240, 10240),
            IntPoint::new(10240, 10240),
            IntPoint::new(10240, -10240)
        ];

        let clip = [
            IntPoint::new(-5120,-5120),
            IntPoint::new(-5120,15360),
            IntPoint::new(15360,15360),
            IntPoint::new(15360,-5120)
        ];

        let mut overlay = Overlay::new(2);
        overlay.add_path(&subj, ShapeType::Subject);
        overlay.add_path(&clip, ShapeType::Clip);

        let shapes = overlay.build_vectors(FillRule::NonZero, OverlayRule::Subject, Solver::Auto);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].len(), 1);

        let vectors = &shapes[0][0];
        let template = [
            VectorEdge {
                a: IntPoint::new(-10240,-10240),
                b: IntPoint::new(-10240,10240),
                fill: 2
            },
            VectorEdge {
                a: IntPoint::new(-10240,10240),
                b: IntPoint::new(-5120,10240),
                fill: 2
            },
            VectorEdge {
                a: IntPoint::new(-5120,10240),
                b: IntPoint::new(10240,10240),
                fill: 14
            },
            VectorEdge {
                a: IntPoint::new(10240,10240),
                b: IntPoint::new(10240,-5120),
                fill: 14
            },
            VectorEdge {
                a: IntPoint::new(10240,-5120),
                b: IntPoint::new(10240,-10240),
                fill: 2
            },
            VectorEdge {
                a: IntPoint::new(10240,-10240),
                b: IntPoint::new(-10240,-10240),
                fill: 2
            }
        ];

        assert_eq!(vectors.as_slice(), template.as_slice());
    }
}