use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString},
};
use fm_core::{Config, Guard, Result, error};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
pub fn normalize(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v) => v.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        _ => ip,
    }
}
pub fn is_admin(ip: IpAddr) -> bool {
    normalize(ip) == IpAddr::V4(Ipv4Addr::LOCALHOST)
}
pub fn parse_mac(value: &str) -> Result<[u8; 6]> {
    let value = value.replace('-', ":");
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 6 {
        return Err(error(
            "MAC-Adresse muss aus sechs Hexadezimalpaaren bestehen",
        ));
    }
    let mut mac = [0; 6];
    for (i, p) in parts.iter().enumerate() {
        if p.len() != 2 {
            return Err(error("Ungültige MAC-Adresse"));
        }
        mac[i] = u8::from_str_radix(p, 16).map_err(|_| error("Ungültige MAC-Adresse"))?;
    }
    Ok(mac)
}
pub fn parse_ip(value: &str) -> Result<ipnet::IpNet> {
    if let Ok(ip) = value.parse::<IpAddr>() {
        return Ok(normalize(ip).into());
    }
    value
        .parse()
        .map_err(|_| error("Ungültige IP-Adresse oder CIDR-Netzmaske"))
}
pub fn validate(config: &Config) -> Result<()> {
    for ip in &config.allowed_ips {
        parse_ip(ip)?;
    }
    for mac in &config.allowed_macs {
        parse_mac(mac)?;
    }
    if config.network_enabled {
        if config.allowed_ips.is_empty() && config.allowed_macs.is_empty() {
            return Err(error(
                "Netzwerkzugriff benötigt mindestens einen Whitelist-Eintrag",
            ));
        }
        if PasswordHash::new(&config.password_hash).is_err() {
            return Err(error("Netzwerkzugriff benötigt ein gültiges Passwort"));
        }
        let addr: SocketAddr = config
            .network_bind
            .parse()
            .map_err(|_| error("Netzwerkschnittstelle als IP:Port eingeben"))?;
        if addr.port() == 8484 || addr.port() == 0 {
            return Err(error(
                "Für HTTPS einen eigenen Port verwenden, zum Beispiel 8485",
            ));
        }
    }
    Ok(())
}
pub fn allowed(ip: IpAddr, config: &Config, mac: Option<[u8; 6]>) -> bool {
    let ip = normalize(ip);
    if is_admin(ip) || ip == IpAddr::V6(std::net::Ipv6Addr::LOCALHOST) {
        return true;
    }
    if !config.network_enabled {
        return false;
    }
    config
        .allowed_ips
        .iter()
        .filter_map(|s| parse_ip(s).ok())
        .any(|net| net.contains(&ip))
        || mac.is_some_and(|m| {
            config
                .allowed_macs
                .iter()
                .filter_map(|s| parse_mac(s).ok())
                .any(|v| v == m)
        })
}
pub fn password_hash(password: &str) -> Result<String> {
    if password.chars().count() < 12 {
        return Err(error(
            "Das Netzwerkpasswort benötigt mindestens zwölf Zeichen",
        ));
    }
    let salt = SaltString::encode_b64(uuid_bytes().as_slice()).map_err(error)?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|p| p.to_string())
        .map_err(error)
}
fn uuid_bytes() -> [u8; 16] {
    *uuid::Uuid::new_v4().as_bytes()
}
pub fn verify(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).ok().is_some_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}
pub fn tls_material(guard: &Guard, bind: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let cert = guard.resolve(".folder-mind/tls-cert.pem")?;
    let key = guard.resolve(".folder-mind/tls-key.pem")?;
    if cert.exists() && key.exists() {
        return Ok((std::fs::read(cert)?, std::fs::read(key)?));
    }
    let mut names = vec!["localhost".into(), "127.0.0.1".into(), "::1".into()];
    if let Ok(addr) = bind.parse::<SocketAddr>()
        && !addr.ip().is_unspecified()
    {
        names.push(addr.ip().to_string());
    }
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(names).map_err(error)?;
    let cert = cert.pem().into_bytes();
    let key = signing_key.serialize_pem().into_bytes();
    guard.write(".folder-mind/tls-cert.pem", &cert)?;
    guard.write(".folder-mind/tls-key.pem", &key)?;
    Ok((cert, key))
}
#[cfg(windows)]
pub fn resolve_mac(ip: IpAddr) -> Result<[u8; 6]> {
    use windows::Win32::{
        NetworkManagement::IpHelper::{FreeMibTable, GetIpNetTable2, MIB_IPNET_TABLE2},
        Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC},
    };
    let ip = normalize(ip);
    let mut table: *mut MIB_IPNET_TABLE2 = std::ptr::null_mut();
    // Windows reserviert die Tabelle; sie wird nach dem Lesen mit FreeMibTable freigegeben.
    unsafe {
        let status = GetIpNetTable2(AF_UNSPEC, &mut table);
        if status.0 != 0 || table.is_null() {
            return Err(error(
                "Windows-Nachbartabelle konnte nicht gelesen werden. Bitte einen IP-Eintrag verwenden.",
            ));
        }
        let rows =
            std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        let mut found = None;
        for row in rows {
            let candidate = if row.Address.si_family == AF_INET {
                Some(IpAddr::V4(Ipv4Addr::from(
                    row.Address.Ipv4.sin_addr.S_un.S_addr.to_ne_bytes(),
                )))
            } else if row.Address.si_family == AF_INET6 {
                Some(IpAddr::V6(std::net::Ipv6Addr::from(
                    row.Address.Ipv6.sin6_addr.u.Byte,
                )))
            } else {
                None
            };
            if candidate.map(normalize) == Some(ip) && row.PhysicalAddressLength == 6 {
                let mut m = [0; 6];
                m.copy_from_slice(&row.PhysicalAddress[..6]);
                found = Some(m);
                break;
            }
        }
        FreeMibTable(table.cast());
        found.ok_or_else(||error("Keine MAC-Adresse in der Windows-Nachbartabelle gefunden. MAC-Freigaben funktionieren nur im gleichen lokalen Netz; über Router oder VPN bitte IP/CIDR verwenden."))
    }
}
#[cfg(not(windows))]
pub fn resolve_mac(_ip: IpAddr) -> Result<[u8; 6]> {
    Err(error(
        "MAC-Auflösung ist nur im Windows-Build verfügbar. Bitte eine IP-Freigabe verwenden.",
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whitelist_and_admin() {
        let mut c = Config {
            network_enabled: true,
            allowed_ips: vec!["192.168.1.0/24".into(), "2001:db8::/32".into()],
            allowed_macs: vec!["AA:BB:CC:DD:EE:FF".into()],
            ..Config::default()
        };
        assert!(allowed("192.168.1.8".parse().unwrap(), &c, None));
        assert!(allowed("2001:db8::1".parse().unwrap(), &c, None));
        assert!(!allowed("10.0.0.1".parse().unwrap(), &c, None));
        assert!(allowed(
            "10.0.0.1".parse().unwrap(),
            &c,
            Some(parse_mac("aa-bb-cc-dd-ee-ff").unwrap())
        ));
        c.network_enabled = false;
        assert!(!allowed("192.168.1.8".parse().unwrap(), &c, None));
        assert!(allowed("127.0.0.1".parse().unwrap(), &c, None));
        assert!(!is_admin("::1".parse().unwrap()));
        assert!(is_admin("::ffff:127.0.0.1".parse().unwrap()));
        assert!(parse_mac("aa:bb").is_err());
        assert!(parse_ip("1.2.3.4/99").is_err());
        assert!(
            validate(&Config {
                network_enabled: true,
                ..Config::default()
            })
            .is_err()
        );
    }
    #[test]
    fn passwords() {
        let h = password_hash("Ein langes Passwort 2026").unwrap();
        assert!(verify("Ein langes Passwort 2026", &h));
        assert!(!verify("falsch", &h));
        assert!(password_hash("kurz").is_err());
    }
    #[test]
    fn certificate_persisted() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        g.mkdir(".folder-mind").unwrap();
        let a = tls_material(&g, "0.0.0.0:8485").unwrap();
        let b = tls_material(&g, "0.0.0.0:8485").unwrap();
        assert_eq!(a, b);
    }
}

#[cfg(windows)]
pub fn local_hour() -> u8 {
    unsafe { windows::Win32::System::SystemInformation::GetLocalTime().wHour as u8 }
}
#[cfg(not(windows))]
pub fn local_hour() -> u8 {
    ((fm_core::now() / 3600) % 24) as u8
}
pub fn in_time_window(start: u8, end: u8, hour: u8) -> bool {
    if start == end {
        return true;
    }
    if start < end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    }
}
#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<()> {
    use windows::{
        Win32::System::Registry::*,
        core::{PCWSTR, w},
    };
    let mut key = HKEY::default();
    unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
        .ok()
        .map_err(|_| error("Autostart-Schlüssel konnte nicht geöffnet werden"))?;
        let result = if enabled {
            let exe = std::env::current_exe()?;
            let value = format!("\"{}\"", exe.display())
                .encode_utf16()
                .chain(std::iter::once(0))
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>();
            RegSetValueExW(key, w!("FolderMind"), None, REG_SZ, Some(&value))
        } else {
            let r = RegDeleteValueW(key, w!("FolderMind"));
            if r.0 == 2 {
                windows::Win32::Foundation::WIN32_ERROR(0)
            } else {
                r
            }
        };
        let _ = RegCloseKey(key);
        result
            .ok()
            .map_err(|_| error("Autostart konnte nicht geändert werden"))
    }
}
#[cfg(not(windows))]
pub fn set_autostart(enabled: bool) -> Result<()> {
    if enabled {
        Err(error("Autostart ist im Windows-Build verfügbar"))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod time_tests {
    #[test]
    fn overnight_window() {
        assert!(super::in_time_window(22, 6, 23));
        assert!(super::in_time_window(22, 6, 2));
        assert!(!super::in_time_window(22, 6, 12));
        assert!(super::in_time_window(0, 24, 23));
    }
}

pub fn certificate_fingerprint(guard: &Guard) -> Result<String> {
    let bytes = std::fs::read(guard.resolve(".folder-mind/tls-cert.pem")?)?;
    let cert = rustls_pemfile::certs(&mut std::io::Cursor::new(bytes))
        .next()
        .ok_or_else(|| error("Zertifikat enthält keine gültigen Zertifikatsdaten"))?
        .map_err(error)?;
    Ok(fm_core::hash(cert.as_ref()))
}
