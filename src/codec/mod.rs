pub mod frame;

/// # Examples
/// ```
/// use zedmq::codec::greeting;
///
/// let g = greeting();
/// let (left, right) = (&g[..=11], &g[12..]);
/// assert_eq!(left, &[0xFFu8, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 3, 0] as &[u8]);
/// assert_eq!(&right[..4], b"NULL" as &[u8]);
/// ```
pub fn greeting() -> [u8; 64] {
    // TODO: Missing a way to specify the security mechanism (currently NULL) and the as-server field (currently false)

    let mut raw = [0u8; 64];

    raw[0] = 0xFF; // signature start
                   // signature padding.
    raw[9] = 0x7F; // signature end
    raw[10] = 3;
    raw[11] = 0;

    // Security
    raw[12] = 0x4E;
    raw[13] = 0x55;
    raw[14] = 0x4C;
    raw[15] = 0x4C;

    raw
}
