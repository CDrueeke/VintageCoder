// SPDX-License-Identifier: GPL-3.0-or-later
//! JSON-RPC mit Content-Length-Rahmen (LSP-Transport).
use serde_json::Value;
use std::io::{BufRead, Write};

pub fn read_message(r: &mut impl BufRead) -> std::io::Result<Option<Value>> {
    let mut len = None;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line)? == 0 { return Ok(None); }
        let line = line.trim_end();
        if line.is_empty() { break; }
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") { len = v.trim().parse::<usize>().ok(); }
    }
    let Some(len) = len else { return Ok(None) };
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(serde_json::from_slice(&buf).ok())
}

pub fn write_message(w: &mut impl Write, v: &Value) -> std::io::Result<()> {
    let body = v.to_string();
    write!(w, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    w.flush()
}
