pub(crate) fn pad_to_256(value: u32) -> u32 {
    value + (256 - (value % 256))
}
