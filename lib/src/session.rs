use std::{cmp::Ordering, str::FromStr};

/// simplified semver [`Version`]
// FIXME: replace this with proper Version from semver crate. Is this possible? Does nix apply
// semver versioning?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Version {
    major: u16,
    minor: u16,
    patch: Option<u16>,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.unwrap_or(0).cmp(&other.patch.unwrap_or(0)))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_number(s: &str) -> Option<u16> {
    s.chars()
        .take_while(|c| c.is_ascii_digit())
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

/// Session info for statix.
///
/// Some lints only apply after a certain version of nix. This struct is used to keep track of the
/// nix version
pub struct SessionInfo {
    nix_version: Version,
}

impl SessionInfo {
    /// construct a new [`SessionInfo`] from a [`Version`]
    pub fn from_version(nix_version: Version) -> Self {
        Self { nix_version }
    }

    /// Retrieve the "semver" [`Version`] of this [`SessionInfo`]
    pub fn version(&self) -> &Version {
        &self.nix_version
    }
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn parse_trivial() {
        let v = "1.6.1".parse::<Version>().ok();
        assert_eq!(
            v,
            Some(Version {
                major: 1,
                minor: 6,
                patch: Some(1)
            })
        )
    }

    #[test]
    fn parse() {
        let v = "2.4pre20211006_53e4794".parse::<Version>().ok();
        assert_eq!(
            v,
            Some(Version {
                major: 2,
                minor: 4,
                patch: None
            })
        )
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
        assert!(v2 > v1);
    }
}
