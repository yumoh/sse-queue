


// Convert a range to a satisfiable range
pub(super) fn to_satisfiable_range(
    from: Option<u64>,
    to: Option<u64>,
    length: u64,
) -> Result<(u64, u64), &'static str> {
    let (start, mut end) = match (from, to) {
        (Some(x), Some(z)) => (x, z),                // FromToAll
        (Some(x), None) => (x, length - 1),          // FromTo
        (None, Some(z)) => (length - z, length - 1), // FromEnd
        (None, None) => return Err("You need at least one value to satisfy a range request"),
    };

    if end < start {
        return Err("A byte-range-spec is invalid if the last-byte-pos value is present and less than the first-byte-pos.");
    }
    if end > length {
        end = length
    }

    Ok((start, end))
}

pub(super) fn range_header_parts(header: &range_header::ByteRange) -> (Option<u64>, Option<u64>) {
    use range_header::ByteRange::{FromTo, FromToAll, Last};
    match *header {
        FromTo(x) => (Some(x), None),
        FromToAll(x, y) => (Some(x), Some(y)),
        Last(x) => (None, Some(x)),
    }
}
