use tokio::{io::AsyncReadExt, net::TcpStream};

// macOS may reset a closed socket after an early rejection. A fully framed
// response is already complete; never accept a reset with missing body bytes.
pub async fn response(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut raw = Vec::new();
    if let Err(error) = stream.read_to_end(&mut raw).await
        && (error.kind() != std::io::ErrorKind::ConnectionReset || !complete(&raw))
    {
        return Err(error);
    }
    Ok(raw)
}
fn complete(raw: &[u8]) -> bool {
    let Some(split) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&raw[..split]);
    let body = &raw[split + 4..];
    for line in headers.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.eq_ignore_ascii_case("content-length") {
            return value.trim().parse::<usize>().is_ok_and(|n| body.len() == n);
        }
        if key.eq_ignore_ascii_case("transfer-encoding")
            && value.trim().eq_ignore_ascii_case("chunked")
        {
            return body.ends_with(b"0\r\n\r\n");
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_reset_never_hides_a_truncated_response() {
        assert!(complete(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc"));
        assert!(!complete(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nab"));
        assert!(complete(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n0\r\n\r\n"
        ));
        assert!(!complete(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc"
        ));
        assert!(!complete(b"HTTP/1.1 200 OK\r\nContent-Length: 3"));
    }
}
