use crate::storage;
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    Debug, Copy, Clone, PartialEq, Serialize, Deserialize, EnumString, EnumIter, IntoStaticStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Os {
    Linux,
    Macos,
    Windows,
    Fedora,
    Ubuntu,
    Debian,
    Arch,
    Centos,
    Rhel,
    Opensuse,
    Mint,
    Bsd,
    Any,
}

impl Os {
    pub fn icon(&self) -> &'static str {
        if storage::use_nerd_fonts() {
            match self {
                Os::Linux => "\u{f17c}",    //
                Os::Macos => "\u{f179}",    //
                Os::Windows => "\u{f17a}",  //
                Os::Fedora => "\u{f30a}",   //
                Os::Ubuntu => "\u{f31b}",   //
                Os::Debian => "\u{f306}",   //
                Os::Arch => "\u{f303}",     //
                Os::Centos => "\u{f304}",   //
                Os::Rhel => "\u{f316}",     //
                Os::Opensuse => "\u{f314}", //
                Os::Mint => "\u{f30e}",     //
                Os::Bsd => "\u{f30c}",      //
                Os::Any => "\u{f484}",      //
            }
        } else {
            match self {
                Os::Macos => "🍎",
                Os::Windows => "🪟",
                Os::Linux
                | Os::Fedora
                | Os::Ubuntu
                | Os::Debian
                | Os::Arch
                | Os::Centos
                | Os::Rhel
                | Os::Opensuse
                | Os::Mint => "🐧",
                Os::Bsd => "😈",
                Os::Any => "🌐",
            }
        }
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strum::IntoEnumIterator;

    #[test]
    fn all_os_variants_have_icons() {
        for os in Os::iter() {
            let icon = os.icon();
            assert!(!icon.is_empty(), "Os variant {:?} has empty icon", os);
        }
    }

    #[test]
    fn os_display_lowercase() {
        assert_eq!(Os::Linux.to_string(), "linux");
        assert_eq!(Os::Macos.to_string(), "macos");
        assert_eq!(Os::Windows.to_string(), "windows");
        assert_eq!(Os::Fedora.to_string(), "fedora");
        assert_eq!(Os::Ubuntu.to_string(), "ubuntu");
        assert_eq!(Os::Debian.to_string(), "debian");
        assert_eq!(Os::Arch.to_string(), "arch");
        assert_eq!(Os::Centos.to_string(), "centos");
        assert_eq!(Os::Rhel.to_string(), "rhel");
        assert_eq!(Os::Opensuse.to_string(), "opensuse");
        assert_eq!(Os::Mint.to_string(), "mint");
        assert_eq!(Os::Bsd.to_string(), "bsd");
        assert_eq!(Os::Any.to_string(), "any");
    }

    #[test]
    fn os_enum_string_parse() {
        assert_eq!(Os::from_str("linux").unwrap(), Os::Linux);
        assert_eq!(Os::from_str("macos").unwrap(), Os::Macos);
        assert_eq!(Os::from_str("windows").unwrap(), Os::Windows);
        assert_eq!(Os::from_str("ubuntu").unwrap(), Os::Ubuntu);
        assert_eq!(Os::from_str("any").unwrap(), Os::Any);
    }

    #[test]
    fn os_serde_deserialize_lowercase() {
        let json: Result<Os, _> = serde_json::from_str(r#""linux""#);
        assert_eq!(json.unwrap(), Os::Linux);

        let json: Result<Os, _> = serde_json::from_str(r#""macos""#);
        assert_eq!(json.unwrap(), Os::Macos);

        let json: Result<Os, _> = serde_json::from_str(r#""any""#);
        assert_eq!(json.unwrap(), Os::Any);
    }

    #[test]
    fn os_icon_non_nerd_mode() {
        let any_icon = Os::Any.icon();
        assert!(!any_icon.is_empty());

        let macos_icon = Os::Macos.icon();
        assert!(!macos_icon.is_empty());

        let bsd_icon = Os::Bsd.icon();
        assert!(!bsd_icon.is_empty());
    }
}
