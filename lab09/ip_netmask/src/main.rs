use std::net::IpAddr;

fn main() {
    let interfaces = if_addrs::get_if_addrs().unwrap_or_else(|e| {
        eprintln!("failed to get interfaces: {e}");
        std::process::exit(1);
    });

    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }
        let ip = iface.ip();
        let netmask: IpAddr = match &iface.addr {
            if_addrs::IfAddr::V4(a) => IpAddr::V4(a.netmask),
            if_addrs::IfAddr::V6(a) => IpAddr::V6(a.netmask),
        };
        let prefix = prefix_len(&netmask);
        println!("{}: ip={} netmask={} (/{prefix})", iface.name, ip, netmask);
    }
}

fn prefix_len(mask: &IpAddr) -> u32 {
    match mask {
        IpAddr::V4(m) => u32::from_be_bytes(m.octets()).count_ones(),
        IpAddr::V6(m) => m.octets().iter().map(|b| b.count_ones()).sum(),
    }
}
