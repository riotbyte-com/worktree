use socket2::{Domain, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};

/// Check if a port is available for binding
pub fn is_port_free(port: u16) -> bool {
    let socket = match Socket::new(Domain::IPV4, Type::STREAM, None) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Note: We intentionally don't use set_reuse_address here.
    // While SO_REUSEADDR would avoid TIME_WAIT issues, it can give false positives
    // by reporting a port as "free" when another process is actively using it.
    // It's better to get accurate availability checks even if it means
    // occasionally skipping ports in TIME_WAIT state.

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    socket.bind(&addr.into()).is_ok()
}

/// Find N consecutive free ports in the given range, excluding already allocated ports
pub fn find_consecutive_free(
    count: u16,
    start: u16,
    end: u16,
    exclude: &std::collections::HashSet<u16>,
) -> Option<Vec<u16>> {
    if count == 0 {
        return Some(vec![]);
    }

    let mut current_start = start;

    while current_start + count <= end {
        let mut all_free = true;
        let mut ports = Vec::with_capacity(count as usize);

        for i in 0..count {
            let port = current_start + i;
            if exclude.contains(&port) || !is_port_free(port) {
                all_free = false;
                current_start = port + 1;
                break;
            }
            ports.push(port);
        }

        if all_free {
            return Some(ports);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_port_free() {
        // High ports should generally be free
        let result = is_port_free(59999);
        // Can't guarantee this, so just check it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_find_consecutive_free_empty() {
        let exclude = std::collections::HashSet::new();
        let result = find_consecutive_free(0, 50000, 50100, &exclude);
        assert_eq!(result, Some(vec![]));
    }
}
