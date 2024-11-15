use bytes::{BufMut, BytesMut};

#[derive(Debug, Default)]
pub(crate) struct NALUSplitter {
    buffer: BytesMut,
    pts: Option<u64>,
}

fn find_nalu_start_code(buf: &[u8]) -> Option<usize> {
    if buf.is_empty() {
        return None;
    };

    buf.windows(3)
        .enumerate()
        .filter(|(_, window)| **window == [0, 0, 1])
        .filter(|(i, window)| !(*i == 0 || (*i == 1 && window[0] == 0)))
        .map(|(i, _)| i + 3)
        .next()
}

impl NALUSplitter {
    pub(crate) fn push(
        &mut self,
        bytestream: &[u8],
        pts: Option<u64>,
    ) -> Vec<(Vec<u8>, Option<u64>)> {
        let mut output_pts = if self.buffer.is_empty() {
            pts
        } else {
            self.pts
        };

        self.buffer.put(bytestream);
        let mut result = Vec::new();

        while let Some(i) = find_nalu_start_code(&self.buffer) {
            let nalu = self.buffer.split_to(i);
            result.push((nalu.to_vec(), output_pts));
            output_pts = pts;
        }

        self.pts = pts;

        result
    }
}
