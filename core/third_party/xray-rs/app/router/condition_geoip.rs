use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cidr {
    pub ip: IpAddr,
    pub prefix: u8,
}

impl Cidr {
    pub fn new(ip: IpAddr, prefix: u8) -> Self {
        Self { ip, prefix }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split('/');
        let ip_str = parts.next()?;
        let ip: IpAddr = ip_str.parse().ok()?;
        let prefix = match parts.next() {
            Some(p) => p.parse::<u8>().ok()?,
            None => match ip {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            },
        };
        Some(Self { ip, prefix })
    }

    pub fn contains(&self, target: &IpAddr) -> bool {
        match (self.ip, target) {
            (IpAddr::V4(net), IpAddr::V4(tgt)) => {
                if self.prefix == 0 {
                    return true;
                }
                if self.prefix > 32 {
                    return false;
                }
                let shift = 32 - self.prefix;
                let mask = if shift >= 32 {
                    0
                } else {
                    !((1u32 << shift) - 1)
                };
                (u32::from_be_bytes(net.octets()) & mask)
                    == (u32::from_be_bytes(tgt.octets()) & mask)
            }
            (IpAddr::V6(net), IpAddr::V6(tgt)) => {
                if self.prefix == 0 {
                    return true;
                }
                if self.prefix > 128 {
                    return false;
                }
                let shift = 128 - self.prefix;
                let mask = if shift >= 128 {
                    0
                } else {
                    !((1u128 << shift) - 1)
                };
                (u128::from_be_bytes(net.octets()) & mask)
                    == (u128::from_be_bytes(tgt.octets()) & mask)
            }
            _ => false,
        }
    }
}

pub trait GeoIpMatcher: Send + Sync {
    fn matches_ip(&self, ip: &IpAddr) -> bool;
    fn any_match(&self, ips: &[IpAddr]) -> bool {
        ips.iter().any(|ip| self.matches_ip(ip))
    }
    fn all_match(&self, ips: &[IpAddr]) -> bool {
        !ips.is_empty() && ips.iter().all(|ip| self.matches_ip(ip))
    }
    fn filter_ips(&self, ips: &[IpAddr]) -> (Vec<IpAddr>, Vec<IpAddr>) {
        let mut matched = Vec::new();
        let mut unmatched = Vec::new();
        for ip in ips {
            if self.matches_ip(ip) {
                matched.push(*ip);
            } else {
                unmatched.push(*ip);
            }
        }
        (matched, unmatched)
    }
}

pub struct HeuristicGeoIpMatcher {
    cidrs: Vec<Cidr>,
    reverse: bool,
}

impl HeuristicGeoIpMatcher {
    pub fn new(cidrs: Vec<Cidr>, reverse: bool) -> Self {
        Self { cidrs, reverse }
    }

    pub fn toggle_reverse(&mut self) {
        self.reverse = !self.reverse;
    }

    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }
}

impl GeoIpMatcher for HeuristicGeoIpMatcher {
    fn matches_ip(&self, ip: &IpAddr) -> bool {
        let is_contained = self.cidrs.iter().any(|net| net.contains(ip));
        if self.reverse {
            !is_contained
        } else {
            is_contained
        }
    }
}
