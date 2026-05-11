use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};

const HEADER_LEN: usize = 12;
const TOMBSTONE: u32 = u32::MAX;

/// Encodes a single log entry into a flat byte buffer.
/// `value = None` writes a delete tombstone.
pub(super) fn encode(key: &[u8], value: Option<&[u8]>) -> Vec<u8> {
    let klen = key.len() as u32;
    let vlen = value.map(|v| v.len() as u32).unwrap_or(TOMBSTONE);
    let vbytes = value.unwrap_or(&[]);

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&klen.to_le_bytes());
    hasher.update(&vlen.to_le_bytes());
    hasher.update(key);
    hasher.update(vbytes);

    let mut buf = Vec::with_capacity(HEADER_LEN + key.len() + vbytes.len());
    buf.extend_from_slice(&hasher.finalize().to_le_bytes());
    buf.extend_from_slice(&klen.to_le_bytes());
    buf.extend_from_slice(&vlen.to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(vbytes);
    buf
}

/// Replays all valid entries from `file` into `index`.
/// Returns the byte offset just past the last valid entry; any bytes beyond
/// that are corrupt or truncated and should be discarded with `set_len`.
pub(super) fn replay(file: &File, index: &mut HashMap<Vec<u8>, Vec<u8>>) -> io::Result<u64> {
    let mut reader = BufReader::new(file);
    let mut good_pos: u64 = 0;
    let mut header = [0u8; HEADER_LEN];

    loop {
        if !fill(&mut reader, &mut header)? {
            break;
        }

        let stored_crc = u32_le(&header[0..]);
        let klen = u32_le(&header[4..]) as usize;
        let vlen_raw = u32_le(&header[8..]);
        let is_delete = vlen_raw == TOMBSTONE;
        let vlen = if is_delete { 0 } else { vlen_raw as usize };

        let mut body = vec![0u8; klen + vlen];
        if !fill(&mut reader, &mut body)? {
            break;
        }

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&header[4..]);
        hasher.update(&body);
        if hasher.finalize() != stored_crc {
            break;
        }

        let key = body[..klen].to_vec();
        if is_delete {
            index.remove(&key);
        } else {
            index.insert(key, body[klen..].to_vec());
        }
        good_pos += (HEADER_LEN + klen + vlen) as u64;
    }

    Ok(good_pos)
}

/// Reads a little-endian `u32` from the start of `src`.
fn u32_le(src: &[u8]) -> u32 {
    u32::from_le_bytes(src[..4].try_into().unwrap())
}

/// Fills `buf` entirely from `reader`.
/// Returns `false` on EOF (whether clean or mid-entry — both mean stop replaying).
fn fill<R: Read>(reader: &mut R, buf: &mut [u8]) -> io::Result<bool> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..])? {
            0 => return Ok(false),
            n => total += n,
        }
    }
    Ok(true)
}
