use std::cmp::Ordering;
use crate::core::overlay::ShapeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeCountBoolean {
    pub subj: i32,
    pub clip: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeCountString {
    pub subj: i32,
    pub clip: u8,
}

pub(crate) const STRING_FORWARD_CLIP: u8 = 0b10;
pub(crate) const STRING_BACK_CLIP: u8 = 0b1;

impl ShapeCountBoolean {
    pub(crate) const SUBJ_DIRECT: ShapeCountBoolean = ShapeCountBoolean { subj: 1, clip: 0 };
    pub(crate) const SUBJ_INVERT: ShapeCountBoolean = ShapeCountBoolean { subj: -1, clip: 0 };
    pub(crate) const CLIP_DIRECT: ShapeCountBoolean = ShapeCountBoolean { subj: 0, clip: 1 };
    pub(crate) const CLIP_INVERT: ShapeCountBoolean = ShapeCountBoolean { subj: 0, clip: -1 };
}

pub(crate) trait WindingCount
where
    Self: Clone + Copy + Send,
{
    fn is_not_empty(&self) -> bool;
    fn new(subj: i32, clip: i32) -> Self;
    fn with_shape_type(shape_type: ShapeType) -> (Self, Self);
    fn add(self, count: Self) -> Self;
    fn apply(&mut self, count: Self);
    fn invert(self) -> Self;
}

impl WindingCount for ShapeCountBoolean {
    #[inline(always)]
    fn is_not_empty(&self) -> bool { self.subj != 0 || self.clip != 0 }

    #[inline(always)]
    fn new(subj: i32, clip: i32) -> Self { Self { subj, clip } }

    #[inline(always)]
    fn with_shape_type(shape_type: ShapeType) -> (Self, Self) {
        match shape_type {
            ShapeType::Subject => (ShapeCountBoolean::SUBJ_DIRECT, ShapeCountBoolean::SUBJ_INVERT),
            ShapeType::Clip => (ShapeCountBoolean::CLIP_DIRECT, ShapeCountBoolean::CLIP_INVERT)
        }
    }

    #[inline(always)]
    fn add(self, count: Self) -> Self {
        let subj = self.subj + count.subj;
        let clip = self.clip + count.clip;

        Self { subj, clip }
    }

    #[inline(always)]
    fn apply(&mut self, count: Self) {
        self.subj += count.subj;
        self.clip += count.clip;
    }

    #[inline(always)]
    fn invert(self) -> Self {
        Self { subj: -self.subj, clip: -self.clip }
    }
}

impl WindingCount for ShapeCountString {
    #[inline(always)]
    fn is_not_empty(&self) -> bool { self.subj != 0 || self.clip != 0 }

    #[inline(always)]
    fn new(subj: i32, clip: i32) -> Self {
        // 0 - bit - back
        // 1 - bit - forward
        let mask = match clip.cmp(&0) {
            Ordering::Greater => STRING_FORWARD_CLIP,
            Ordering::Less => STRING_BACK_CLIP,
            Ordering::Equal => 0,
        };
        Self { subj, clip: mask }
    }

    #[inline(always)]
    fn with_shape_type(shape_type: ShapeType) -> (Self, Self) {
        // todo!!! remove
        match shape_type {
            ShapeType::Subject => (Self { subj: 1, clip: 0 }, Self { subj: -1, clip: 0 }),
            ShapeType::Clip => (Self { subj: 0, clip: STRING_FORWARD_CLIP }, Self { subj: 0, clip: STRING_BACK_CLIP })
        }
    }

    #[inline(always)]
    fn add(self, count: Self) -> Self {
        let subj = self.subj + count.subj;
        let clip = self.clip | count.clip;

        Self { subj, clip }
    }

    #[inline(always)]
    fn apply(&mut self, count: Self) {
        self.subj += count.subj;
        self.clip |= count.clip;
    }

    #[inline(always)]
    fn invert(self) -> Self {
        let b0 = self.clip & 0b01;
        let b1 = self.clip & 0b10;
        let clip = (b0 << 1) | (b1 >> 1);

        Self { subj: -self.subj, clip }
    }
}