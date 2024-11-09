use std::cmp::Ordering;
use crate::geom::x_segment::XSegment;

#[derive(Debug, Clone)]
pub(super) struct CountSegment<C> {
    pub(super) count: C,
    pub(super) x_segment: XSegment,
}

impl<C> Eq for CountSegment<C> {}

impl<C> PartialEq<Self> for CountSegment<C> {
    fn eq(&self, other: &Self) -> bool {
        self.x_segment == other.x_segment
    }
}

impl<C> PartialOrd<Self> for CountSegment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C> Ord for CountSegment<C> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.x_segment.is_under_segment(&other.x_segment) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}