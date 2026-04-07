/// A B+tree node backed by a fixed-size page.
///
/// Internal nodes store keys and child pointers. Leaf nodes store key-value
/// pairs. All nodes are encoded into a flat byte slice with the layout:
///
/// ```text
/// | type (2B) | nkeys (2B) | pointers (nkeys * 8B) | offsets (nkeys * 2B) | KV pairs |
/// ```
///
/// This mirrors the on-disk representation directly, so a node can be written
/// to or read from a page without copying.
pub struct BTreeNode {
    // Backing store for this node's page data. Owned or borrowed from a
    // page cache depending on context.
    #[allow(dead_code)] // placeholder until node encoding is implemented
    data: Vec<u8>,
}
