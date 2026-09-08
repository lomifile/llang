pub fn usize_to_u32(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
