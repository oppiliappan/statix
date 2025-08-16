use std::{cmp::Ordering, str::FromStr};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Version {
    major: u16,
    minor: u16,
    patch: Option<u16>,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let score = |v: &Version| v.major * 100 + v.minor * 10 + v.patch.unwrap_or(0);
        score(self).cmp(&score(other))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_number(s: &str) -> Option<u16> {
    s.chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<u16>()
        .ok()
}

fn parse_version(s: &str) -> Option<Version> {
    let mut parts = s.split('.');
    let major = parse_number(parts.next()?)?;
    let minor = parse_number(parts.next()?)?;
    let patch = parts.next().and_then(parse_number);
    Some(Version {
        major,
        minor,
        patch,
    })
}

impl FromStr for Version {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_version(s).ok_or(())
    }
}

#[non_exhaustive]
pub struct SessionInfo {
    nix_version: Version,
}

impl SessionInfo {
    #[must_use]
    pub fn from_version(nix_version: Version) -> Self {
        Self { nix_version }
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.nix_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trivial() {
        let v = "1.6.1".parse::<Version>().ok();
        assert!(v.is_some());
    }

    #[test]
    fn parse() {
        let v = "2.4pre20211006_53e4794".parse::<Version>().ok();
        assert!(v.is_some());
    }

    #[test]
    fn compare_trivial() {
        let v1 = "1.6.1".parse::<Version>().ok();
        let v2 = "1.7.2".parse::<Version>().ok();
        assert!(v2 > v1);
    }

    #[test]
    fn compare() {
        let v1 = "1.7".parse::<Version>().ok();
        let v2 = "2.4pre20211006_53e4794".parse::<Version>().ok();
        assert!(v2 >= v1);
    }
}
