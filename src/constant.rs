// 2 * max(domains) + edata + hdrs_len + len_bytes(edata) = MTU
pub const MTU: usize = 1600;
pub const MAX_DOMAIN_LEN: usize = 26;
pub const DNS_HDRS_LEN: usize = 26;
pub const RAW_PAYLOAD_LEN: usize = MTU - 2 * MAX_DOMAIN_LEN - DNS_HDRS_LEN;
pub const PAYLOAD_LEN_BYTES: usize = RAW_PAYLOAD_LEN / 256 + 1; // one length byte on each 256 bytes
pub const MAX_PAYLOAD_LEN: usize = RAW_PAYLOAD_LEN - PAYLOAD_LEN_BYTES;
pub const MAX_DATA_LEN: usize = (MAX_PAYLOAD_LEN - 4) * 3 / 4; // base64 encoding overhead
