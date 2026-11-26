//! Atlas: build-time embedded custom client settings for this fork.
//!
//! The JSON payload has the same shape as a signed `custom.txt`, but it is compiled in for the
//! variant named by the `ATLAS_VARIANT` environment variable, so it needs no signature and a
//! `custom.txt` next to the executable cannot replace it.

const DESTEK_JSON: &str = include_str!("../res/atlas/destek.json");
const BAGLANTI_JSON: &str = include_str!("../res/atlas/baglanti.json");

#[derive(Debug, PartialEq)]
pub(crate) enum CustomClientSource {
    Embedded(&'static str),
    File,
}

pub(crate) fn variant_json(name: &str) -> Option<&'static str> {
    match name {
        "destek" => Some(DESTEK_JSON),
        "baglanti" => Some(BAGLANTI_JSON),
        _ => None,
    }
}

/// The payload of the variant this binary was built for; `None` keeps stock RustDesk behavior.
pub(crate) fn embedded_config() -> Option<&'static str> {
    variant_json(option_env!("ATLAS_VARIANT").unwrap_or_default())
}

pub(crate) fn custom_client_source(embedded: Option<&'static str>) -> CustomClientSource {
    match embedded {
        Some(json) => CustomClientSource::Embedded(json),
        None => CustomClientSource::File,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(name: &str) -> serde_json::Value {
        serde_json::from_str(variant_json(name).expect("variant exists")).expect("valid json")
    }

    #[test]
    fn destek_receives_only_with_server_and_key_locked() {
        let v = parsed("destek");
        assert_eq!(v["app-name"], "Atlas Destek");
        assert_eq!(v["conn-type"], "incoming");
        let o = &v["override-settings"];
        assert_eq!(o["custom-rendezvous-server"], "rustdesk.atlantik.tr");
        assert_eq!(o["relay-server"], "rustdesk.atlantik.tr");
        assert_eq!(o["key"], "ITPXTrZ5mbvvLRzWnGfW9KcYWgJrXKnL0eQd9TO9g0w=");
        assert_eq!(o["approve-mode"], "");
        assert_eq!(o["verification-method"], "use-temporary-password");
    }

    #[test]
    fn baglanti_connects_only_with_same_server() {
        let v = parsed("baglanti");
        assert_eq!(v["app-name"], "Atlas Bağlantı");
        assert_eq!(v["conn-type"], "outgoing");
        assert_eq!(
            v["override-settings"]["custom-rendezvous-server"],
            "rustdesk.atlantik.tr"
        );
        assert_eq!(
            v["override-settings"]["key"],
            parsed("destek")["override-settings"]["key"]
        );
    }

    #[test]
    fn both_variants_lock_settings_and_updates() {
        for name in ["destek", "baglanti"] {
            let v = parsed(name);
            for k in [
                "disable-settings",
                "disable-ab",
                "disable-account",
                "disable-installation",
            ] {
                assert_eq!(v[k], "Y", "{name}: {k}");
            }
            let o = &v["override-settings"];
            assert_eq!(o["enable-check-update"], "N", "{name}");
            assert_eq!(o["allow-auto-update"], "N", "{name}");
            assert_eq!(o["enable-clipboard"], "Y", "{name}");
            assert_eq!(o["enable-file-transfer"], "Y", "{name}");
        }
    }

    #[test]
    fn unknown_variant_has_no_config() {
        assert!(variant_json("").is_none());
        assert!(variant_json("rustdesk").is_none());
    }

    #[test]
    fn embedded_config_ignores_custom_txt() {
        let json = variant_json("destek").unwrap();
        assert_eq!(
            custom_client_source(Some(json)),
            CustomClientSource::Embedded(json)
        );
        assert_eq!(custom_client_source(None), CustomClientSource::File);
    }
    // No test applies a payload to the global config here: tests share one process, and
    // `conn-type=incoming` would leak into the client login tests running alongside.
}
