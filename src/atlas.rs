//! Atlas: build-time embedded custom client settings for this fork.

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

    #[test]
    fn applying_destek_sets_app_name_and_hard_settings() {
        // Only test that touches the global config statics.
        crate::common::apply_custom_client_json(variant_json("destek").unwrap().as_bytes());
        assert_eq!(
            hbb_common::config::APP_NAME.read().unwrap().as_str(),
            "Atlas Destek"
        );
        let hard = hbb_common::config::HARD_SETTINGS.read().unwrap();
        assert_eq!(hard.get("conn-type").map(String::as_str), Some("incoming"));
        assert_eq!(hard.get("disable-settings").map(String::as_str), Some("Y"));
    }
}
