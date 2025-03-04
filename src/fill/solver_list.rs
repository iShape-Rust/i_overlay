use std::cmp::Ordering;
use i_float::int::point::IntPoint;
use i_float::triangle::Triangle;
use crate::fill::count_segment::CountSegment;
use crate::fill::solver::{FillSolver, FillStrategy};
use crate::geom::end::End;
use crate::segm::segment::{Segment, SegmentFill, NONE};
use crate::segm::winding_count::WindingCount;
use crate::util::log::Int;

struct ScanFillList<C> {
    buffer: Vec<CountSegment<C>>,
}

impl<C: WindingCount> ScanFillList<C> {
    #[inline(always)]
    fn new(count: usize) -> Self {
        Self { buffer: Vec::with_capacity(count.log2_sqrt()) }
    }

    #[inline(always)]
    fn clear(&mut self, x: i32) {
        self.buffer.retain(|s| s.x_segment.b.x > x);
    }

    #[inline(always)]
    fn insert(&mut self, segment: CountSegment<C>) {
        match self.buffer.binary_search(&segment) {
            Ok(_) => unreachable!("Buffer can only contain unique elements"),
            Err(index) => self.buffer.insert(index, segment)
        }
    }

    #[inline(always)]
    fn find_under_and_nearest(&mut self, p: IntPoint) -> C {
        match self.buffer.binary_search_by(|s|
        if s.x_segment.is_under_point(p) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
        ) {
            Ok(_) => unreachable!("This condition should never occur"),
            Err(index) => {
                if index == 0 {
                    C::new(0, 0)
                } else {
                    unsafe { self.buffer.get_unchecked(index - 1) }.count
                }
            }
        }
    }
}

impl FillSolver {
    pub(super) fn list_fill<F: FillStrategy<C>, C: WindingCount>(segments: &[Segment<C>]) -> Vec<SegmentFill> {
        // Mark. self is sorted by x_segment.a
        let mut scan_list = ScanFillList::new(segments.len());
        let mut buf = Vec::with_capacity(4);

        let n = segments.len();
        let mut result = vec![NONE; n];
        let mut i = 0;

        let mut x0 = 0;

        while i < n {
            let p = segments[i].x_segment.a;
            if p.x != x0 {
                scan_list.clear(p.x);
                x0 = p.x;
            }

            buf.push(End { index: i, point: segments[i].x_segment.b });
            i += 1;

            while i < n && segments[i].x_segment.a == p {
                buf.push(End { index: i, point: segments[i].x_segment.b });
                i += 1;
            }

            buf.sort_by(|s0, s1|
            if Triangle::is_clockwise_point(p, s1.point, s0.point) {
                Ordering::Less
            } else {
                Ordering::Greater
            });

            let mut sum_count = scan_list.find_under_and_nearest(p);
            let mut fill: SegmentFill;

            for se in buf.iter() {
                let sid = unsafe { segments.get_unchecked(se.index) };
                (sum_count, fill) = F::add_and_fill(sid.count, sum_count);
                *unsafe { result.get_unchecked_mut(se.index) } = fill;
                if sid.x_segment.is_not_vertical() {
                    scan_list.insert(CountSegment { count: sum_count, x_segment: sid.x_segment });
                }
            }

            buf.clear();
        }

        result
    }
}
