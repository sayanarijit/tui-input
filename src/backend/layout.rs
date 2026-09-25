use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::input::{codepoint_to_byte, widths};

#[cfg(test)]
mod tests;

/// The visible part of an input, split around the cursor.
pub(crate) struct Line<'a> {
    pub before: &'a str,
    /// The grapheme under the cursor, or a space at the end of the value.
    pub cursor: &'a str,
    pub after: &'a str,
    /// Columns of blank padding after `after`.
    pad: usize,
}

impl Line<'_> {
    /// The blank padding after `after`.
    pub fn padding(&self) -> String {
        " ".repeat(self.pad)
    }
}

/// Fits `value` into `width` display columns, keeping the cursor visible
/// and never splitting a wide grapheme.  One column is reserved to the
/// right of the text so the cursor can sit at the end.
pub(crate) fn layout(value: &str, cursor: usize, width: u16) -> Line<'_> {
    let width = width.max(1) as usize;
    // Split the value at the cursor.  `cursor` counts codepoints, so it is
    // usually but not necessarily on a grapheme boundary.  If
    // `InputRequest::SetCursor` put it inside a grapheme, the split breaks
    // that grapheme in two, and the cursor shows the second half.
    let (head, tail) = value.split_at(codepoint_to_byte(value, cursor));

    // The head gets only the space the tail doesn't need.  This scrolls
    // only as far as needed to keep the cursor visible and keeps the end
    // of the value in view when possible.  If the tail alone overflows,
    // `room` is 0 and the cursor sits at the left edge.  The `- 1`
    // reserves a column for the cursor when it is past the end.
    let room = (width - 1).saturating_sub(widths(tail).sum());

    // Take the grapheme under the cursor off the front of the tail.  Past
    // the end of the value, the cursor is drawn on a space.
    let (cursor, tail) = match tail.graphemes(true).next() {
        Some(g) => (g, &tail[g.len()..]),
        None => (" ", ""),
    };

    // Fit the head from its end, so the text nearest the cursor stays
    // visible.
    let (len, used) = fit(head.graphemes(true).rev(), room);
    let before = &head[head.len() - len..];

    // The rest of the tail gets the columns not used by the head and the
    // cursor.  This saturates only when the cursor grapheme alone is
    // wider than `width`, e.g. a 2-column emoji at width 1.  It then
    // overflows by a column, which is better than not drawing the cursor.
    let room = width.saturating_sub(used + cursor.width());

    // Fit the tail from its start, and pad any columns left over.  A wide
    // grapheme that doesn't fit at the edge is omitted.
    let (len, used) = fit(tail.graphemes(true), room);

    Line {
        before,
        cursor,
        after: &tail[..len],
        pad: room - used,
    }
}

/// Returns the byte length and width of the leading `graphemes` that fit
/// in `room` columns.
fn fit<'a>(graphemes: impl Iterator<Item = &'a str>, room: usize) -> (usize, usize) {
    let (mut len, mut used) = (0, 0);
    for g in graphemes {
        let w = g.width();
        if used + w > room {
            break;
        }
        len += g.len();
        used += w;
    }
    (len, used)
}
