use crate::os::Os;
use std::fs;

fn detect_os_from_os_release(content: &str) -> Os {
    for line in content.lines() {
        if let Some(id) = line.strip_prefix("ID=") {
            let id = id.trim_matches('"').to_lowercase();
            return match id.as_str() {
                "fedora" => Os::Fedora,
                "ubuntu" => Os::Ubuntu,
                "debian" => Os::Debian,
                "arch" => Os::Arch,
                "centos" => Os::Centos,
                "rhel" => Os::Rhel,
                "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => Os::Opensuse,
                "linuxmint" => Os::Mint,
                _ => Os::Linux,
            };
        }
    }
    Os::Linux
}

fn detect_linux_distro() -> Os {
    let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
    detect_os_from_os_release(&content)
}

pub fn detect_os() -> Os {
    match std::env::consts::OS {
        "macos" => Os::Macos,
        "windows" => Os::Windows,
        "linux" => detect_linux_distro(),
        "freebsd" | "openbsd" | "netbsd" => Os::Bsd,
        _ => Os::Linux,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_empty_content() {
        assert_eq!(detect_os_from_os_release(""), Os::Linux);
    }

    #[test]
    fn detect_no_id_line_fallback_to_linux() {
        let content = r#"NAME=NoId
VERSION=1.0
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Linux);
    }

    #[test]
    fn detect_unknown_fallback_to_linux() {
        let content = r#"NAME=SomeUnknown
ID=someunknownos
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Linux);
    }

    #[test]
    fn detect_id_with_quotes() {
        let content = r#"ID="Ubuntu""#;
        assert_eq!(detect_os_from_os_release(content), Os::Ubuntu);
    }

    #[test]
    fn detect_fedora() {
        let content = r#"NAME=Fedora
VERSION="39 (Workstation Edition)"
ID=fedora
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Fedora);
    }

    #[test]
    fn detect_ubuntu() {
        let content = r#"NAME="Ubuntu"
VERSION="22.04.3 LTS"
ID=ubuntu
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Ubuntu);
    }

    #[test]
    fn detect_debian() {
        let content = r#"PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
ID=debian
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Debian);
    }

    #[test]
    fn detect_arch() {
        let content = r#"NAME=Arch Linux
ID=arch
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Arch);
    }

    #[test]
    fn detect_centos() {
        let content = r#"NAME="CentOS Linux"
ID=centos
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Centos);
    }

    #[test]
    fn detect_rhel() {
        let content = r#"NAME="Red Hat Enterprise Linux"
ID=rhel
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Rhel);
    }

    #[test]
    fn detect_opensuse_tumbleweed() {
        let content = r#"NAME=openSUSE Tumbleweed
ID=opensuse-tumbleweed
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Opensuse);
    }

    #[test]
    fn detect_opensuse_leap() {
        let content = r#"NAME="openSUSE Leap 15.6"
ID=opensuse-leap
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Opensuse);
    }

    #[test]
    fn detect_mint() {
        let content = r#"NAME=Ubuntu
ID=linuxmint
"#;
        assert_eq!(detect_os_from_os_release(content), Os::Mint);
    }
}
