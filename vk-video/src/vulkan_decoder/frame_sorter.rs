use std::collections::BinaryHeap;

struct FrameWithPts<T> {
    frame: T,
    pic_order_cnt: i32,
    pts: Option<u64>,
}

impl<T> PartialEq for FrameWithPts<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pic_order_cnt.eq(&other.pic_order_cnt)
    }
}

impl<T> Eq for FrameWithPts<T> {}

impl<T> PartialOrd for FrameWithPts<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.pic_order_cnt.cmp(&other.pic_order_cnt).reverse())
    }
}

impl<T> Ord for FrameWithPts<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pic_order_cnt.cmp(&other.pic_order_cnt).reverse()
    }
}


#[derive(Default)]
pub(crate) struct FrameSorter<T> {
    frames: BinaryHeap<FrameWithPts<T>>
}

impl<T> FrameSorter<T> {
    pub(crate) fn put(&mut self, frame: T, pts: Option<u64>, pic_order_cnt: i32, max_num_reorder_frames: usize) -> Vec<(T, Option<u64>)> {
        self.frames.push(FrameWithPts { frame, pts, pic_order_cnt });

        let mut result = Vec::new();

        while self.frames.len() > max_num_reorder_frames {
            let frame = self.frames.pop().unwrap();

            result.push((frame.frame, frame.pts));
        }

        result
    }
}
