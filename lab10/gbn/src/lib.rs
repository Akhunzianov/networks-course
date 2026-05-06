pub const PKT_DATA: u8 = 0;
pub const PKT_ACK: u8 = 1;
pub const PKT_FIN: u8 = 2;
pub const PKT_FIN_ACK: u8 = 3;

pub const MAX_PAYLOAD: usize = 1024;
pub const HEADER_LEN: usize = 1 + 4 + 2;

pub fn encode(kind: u8, seq: u32, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_LEN + payload.len());
    buf.push(kind);
    buf.extend_from_slice(&seq.to_le_bytes());
    buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    buf.extend_from_slice(payload);
    buf
}

pub fn decode(buf: &[u8]) -> Option<(u8, u32, &[u8])> {
    if buf.len() < HEADER_LEN {
        return None;
    }
    let kind = buf[0];
    let seq = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
    let len = u16::from_le_bytes([buf[5], buf[6]]) as usize;
    if buf.len() < HEADER_LEN + len {
        return None;
    }
    Some((kind, seq, &buf[HEADER_LEN..HEADER_LEN + len]))
}
