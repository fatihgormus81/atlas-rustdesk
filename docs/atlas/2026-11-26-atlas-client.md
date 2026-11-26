# Atlas (RustDesk tabanlı istemci) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** RustDesk 1.5.3'ün fork'undan, sunucu/anahtar/kilitleri ve Atlantik logosunu içine gömülü iki
taşınabilir Windows exe üretmek: `AtlasDestek.exe` (yalnızca gelen) ve `AtlasBaglanti.exe` (yalnızca giden).

**Architecture:** Pro'nun imzalı `custom.txt` yükü (app-name, override-settings, hard settings) aynı
biçimde JSON olarak repoya konur ve derleme anında `ATLAS_VARIANT` ortam değişkenine göre
`include_str!` ile gömülür; `load_custom_client()` gömülü yük varsa onu imzasız uygular ve
`custom.txt`'e hiç bakmaz. Görüntü/taşıma koduna dokunulmaz. Derleme, upstream Windows işinden
türetilmiş tek bir GitHub Actions iş akışında iki varyantlı matrisle yapılır.

**Tech Stack:** Rust (librustdesk), Flutter 3.24.5 Windows runner, upstream `build.py` ve
`libs/portable/generate.py`, GitHub Actions, ImageMagick (ikon üretimi). Site: Laravel 13 / Pest.

**Spec:** `docs/superpowers/specs/2026-11-26-atlas-client-design.md` (Task 1'de fork'a
`docs/atlas/` altına taşınır).

## Global Constraints

- Upstream taban: `rustdesk/rustdesk` etiketi `1.5.3`; alt modül `libs/hbb_common` @ `fae47e024dd83cef6aaecde0aa7d11731df11649`, fork'lanmaz, değiştirilmez.
- Fork: `fatihgormus81/atlas-rustdesk`, herkese açık. Yerel klon `D:\HERD\atlas-rustdesk`. Çalışma dalı `atlas`, `master`'a dokunulmaz.
- Sunucu: `rustdesk.atlantik.tr` (rendezvous ve relay). Anahtar: `ITPXTrZ5mbvvLRzWnGfW9KcYWgJrXKnL0eQd9TO9g0w=`. `api-server` boş.
- Varyantlar: `destek` → app-name `Atlas Destek`, exe `AtlasDestek.exe`, `conn-type=incoming`; `baglanti` → app-name `Atlas Bağlantı`, exe `AtlasBaglanti.exe`, `conn-type=outgoing`.
- İki varyantta: `disable-settings=Y`, `disable-ab=Y`, `disable-account=Y`, `disable-installation=Y`, `enable-check-update=N`, `allow-auto-update=N`, pano ve dosya aktarımı açık.
- Destek onayı: `approve-mode` = `""` (şifre ya da tık, ikisi de), `verification-method` = `use-temporary-password`. Tek kullanımlık şifre her yetkili girişte zaten yenileniyor (`src/server/connection.rs:1913`), yama yok.
- Yetki: açılışta UAC yok; oturum içi yükseltme korunur (`hide-elevate-button-in-accept-window` verilmez).
- Görüntü, kodek, taşıma, `hbb_common` kodunda değişiklik yok. Upstream iş akışı dosyaları silinmez.
- İmzasız exe; MSI yok; Mac/Linux yok.
- Exe meta verisi: CompanyName `Atlantik Yazılım`, ProductName `Atlas`, FileDescription `Atlas — RustDesk tabanlı uzak destek`, LegalCopyright `© 2026 Atlantik Yazılım · RustDesk tabanlıdır (AGPL-3.0)`. Kaynak dosyalardaki RustDesk telif bildirimleri ve `LICENCE` korunur.
- Sürüm adı `1.5.3-atlas.N`; etiket `atlas-v1.5.3-atlas.N`.
- Fork'ta commit mesajları ve kod yorumları İngilizce, emir kipi, prefix yok, sonunda `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. atlantik-website'te aynı kural (repo İngilizce).
- Kullanıcıya görünen yeni metinler Türkçe.

## Review Focus

1. **Aynı bilgisayarda iki Atlas + resmi RustDesk:** taşınabilir paketleyici açma klasörünü içteki exe adından türetir (`libs/portable/src/main.rs:31` `app_dir_name`). İçteki exe `rustdesk.exe` kalırsa üçü `%LOCALAPPDATA%\rustdesk`'te çakışır. → Task 2'de CI adımı paket içindeki exe adını doğrular; Task 4 elle test 6.
2. **Exe yanına konan `custom.txt`:** gömülü ayarı ezmemeli. → Task 2 birim testi `embedded_config_ignores_custom_txt`.
3. **`Atlas Bağlantı` adındaki `ğ`:** ayar klasörü, IPC pipe adı, servis adı bu addan türer. → Task 2 elle doğrulama adımı; sorun olursa iç ad `AtlasBaglanti` (görünen ad değişmez).
4. **Sunucu boş/ulaşılamaz:** `custom-rendezvous-server` boş kalırsa istemci `rs-ny.rustdesk.com`'a düşer (hbb_common `RENDEZVOUS_SERVERS`). → Task 2 testi her varyantta sunucu ve anahtarın boş olmadığını doğrular; Task 4 elle test 7 (ağ izleme).
5. **`disable-installation=Y` iken oturum içi yetki yükseltme:** taşınabilir servis yolu kapanmamalı. → Task 4 elle test 3b; kapanıyorsa Task 2'ye geri dönülür (kurulum düğmesi başka yoldan gizlenir).

---

### Task 1: Fork, çalışma dalı ve sadeleştirilmiş Windows derleme hattı

Hedef: fork'ta **değiştirilmemiş** 1.5.3'ü kendi iş akışımızla derleyip imzasız taşınabilir exe almak.
Bu task hattın kendisini doğrular; marka/ayar yok.

**Files:**
- Create: `.github/workflows/atlas-windows.yml`
- Create: `docs/atlas/2026-11-26-atlas-client-design.md` (spec'in kopyası), `docs/atlas/2026-11-26-atlas-client.md` (bu plan)

**Interfaces:**
- Produces: `atlas-windows.yml` içinde `build-windows` işi, `matrix.variant` alanına hazır (Task 2 doldurur); çıktı artifact adı `atlas-windows-${{ matrix.variant }}`; paketleme adımı `PACK_EXE` ortam değişkeninden exe adını alır (Task 1'de `rustdesk.exe`).

- [ ] **Step 1: Kullanıcı adımı — fork.** Kullanıcıdan iste: github.com/rustdesk/rustdesk → Fork → ad `atlas-rustdesk`, "Copy the master branch only" **işaretsiz** (etiketler gerekli). Fork'un Actions sekmesinde "I understand my workflows, go ahead and enable them". Ardından upstream iş akışlarının tamamını Actions → her iş akışı → "Disable workflow" ile kapat (dosyalar silinmez). Kontrol: `git ls-remote --tags https://github.com/fatihgormus81/atlas-rustdesk.git 1.5.3` bir satır döner.

- [ ] **Step 2: Klonla ve dalı aç.**

```powershell
git clone https://github.com/fatihgormus81/atlas-rustdesk.git D:\HERD\atlas-rustdesk
cd D:\HERD\atlas-rustdesk
git remote add upstream https://github.com/rustdesk/rustdesk.git
git checkout -b atlas 1.5.3
git submodule update --init libs/hbb_common
```
Kontrol: `git -C libs/hbb_common rev-parse HEAD` → `fae47e024dd83cef6aaecde0aa7d11731df11649`.

- [ ] **Step 3: Spec ve planı `docs/atlas/`'a kopyala.** Kaynak: `D:\HERD\rustdesk\docs\superpowers\specs\2026-11-26-atlas-client-design.md` ve bu plan. İç ağ adresi (192.168.x) içermediğini `Select-String -Pattern '192\.168'` ile doğrula (boş dönmeli).

- [ ] **Step 4: `atlas-windows.yml` yaz.** Upstream `.github/workflows/flutter-build.yml`'den yalnızca şunları al: üst `env` bloğundan `RUST_VERSION`, `LLVM_VERSION`, `FLUTTER_VERSION`, `VCPKG_COMMIT_ID`, `VCPKG_BINARY_SOURCES`, `VERSION`; `generate-bridge` işi (satır 131–156, `upload-artifact: true` sabit); `build-for-windows-flutter` işinin x64 matris satırı ve adımları "Checkout"tan "Run the Rust tests"e kadar (LLVM, Flutter, özel engine, Flutter yaması, Rust, vcpkg, `python3 .\build.py --portable --flutter --skip-portable-pack --hwcodec`, sürücü/usbmmidd kopyalama, `cargo test --locked --release --lib --features "$FEATURES"`), "find Runner.res" ve "Build self-extracted executable" (satır ~466–480). Atla: arm64, sciter, imzalama, şablon paketleyici, MSI, TopMostWindow, güncelleme meta verisi, SBOM. Tetikleyici: `workflow_dispatch` ve `push: tags: ['atlas-v*']`. Matris: `variant: [destek]` (Task 2 `baglanti`'yı ekler). Paketleme `generate.py -f ../../rustdesk/ -o . -e ../../rustdesk/$PACK_EXE`; çıktı `atlas-windows-${{ matrix.variant }}` artifact'ı olarak yüklenir.

- [ ] **Step 5: Commit ve push.**

```powershell
git add .github/workflows/atlas-windows.yml docs/atlas
git commit -m "Add a Windows-only build workflow for Atlas" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -u origin atlas
```

- [ ] **Step 6: Hattı çalıştır.** Actions → "Atlas Windows" → Run workflow (dal `atlas`). Beklenen: iş yeşil, `cargo test` geçer, artifact'ta tek bir taşınabilir exe. Kırmızıysa kopyalanan adımlarda eksik ortam değişkeni/önbellek ara; upstream kaynağa dokunma.

- [ ] **Step 7: Duman testi.** Artifact'taki exe temiz bir Windows'ta açılır, resmi RustDesk arayüzü gelir (ayar gömülü değil — beklenen). Sonuç: hat çalışıyor.

---

### Task 2: Gömülü ayar ve iki varyant

**Files:**
- Create: `res/atlas/destek.json`, `res/atlas/baglanti.json`
- Create: `src/atlas.rs`
- Modify: `src/lib.rs` (`mod atlas;` ekle)
- Modify: `src/common.rs` — `read_custom_client` (satır 2607) ve `load_custom_client` (satır 2509)
- Modify: `.github/workflows/atlas-windows.yml`

**Interfaces:**
- Consumes: Task 1'deki `build-windows` işi, `matrix.variant`, `PACK_EXE`.
- Produces:
  - `crate::atlas::embedded_config() -> Option<&'static str>` — derleme anındaki `option_env!("ATLAS_VARIANT")` `"destek"`/`"baglanti"` ise ilgili JSON, aksi hâlde `None` (stok davranış).
  - `crate::atlas::variant_json(name: &str) -> Option<&'static str>` — iki JSON'u da `include_str!` ile taşır; testler ve `embedded_config` bunu kullanır.
  - `crate::common::apply_custom_client_json(data: &[u8])` — `read_custom_client`'ın imza doğrulamasından **sonraki** gövdesi (app-name, default/override, kalan anahtarlar → `HARD_SETTINGS`) buraya taşınır; `read_custom_client` doğruladıktan sonra bunu çağırır, davranışı değişmez.
  - `crate::atlas::CustomClientSource` (`#[derive(Debug, PartialEq)] enum { Embedded(&'static str), File }`) ve `crate::atlas::custom_client_source(embedded: Option<&'static str>) -> CustomClientSource` — `Some` → `Embedded`, `None` → `File`. `load_custom_client()` buna göre dallanır; `Embedded` iken `custom.txt` (debug `./custom.txt` dahil) hiç okunmaz.
  - Varyant exe adları: `destek` → `AtlasDestek.exe`, `baglanti` → `AtlasBaglanti.exe` (iş akışında `PACK_EXE`).

**JSON içeriği** (`destek.json`; `baglanti.json` aynı, yalnızca `app-name` `Atlas Bağlantı`, `conn-type` `outgoing`, `approve-mode`/`verification-method` yok):

```json
{
  "app-name": "Atlas Destek",
  "override-settings": {
    "custom-rendezvous-server": "rustdesk.atlantik.tr",
    "relay-server": "rustdesk.atlantik.tr",
    "api-server": "",
    "key": "ITPXTrZ5mbvvLRzWnGfW9KcYWgJrXKnL0eQd9TO9g0w=",
    "approve-mode": "",
    "verification-method": "use-temporary-password",
    "enable-clipboard": "Y",
    "enable-file-transfer": "Y",
    "enable-check-update": "N",
    "allow-auto-update": "N"
  },
  "conn-type": "incoming",
  "disable-settings": "Y",
  "disable-ab": "Y",
  "disable-account": "Y",
  "disable-installation": "Y"
}
```

- [ ] **Step 1: Başarısız testleri yaz** (`src/atlas.rs` içinde `#[cfg(test)] mod tests`):

```rust
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
    assert_eq!(v["override-settings"]["custom-rendezvous-server"], "rustdesk.atlantik.tr");
    assert_eq!(v["override-settings"]["key"], parsed("destek")["override-settings"]["key"]);
}

#[test]
fn both_variants_lock_settings_and_updates() {
    for name in ["destek", "baglanti"] {
        let v = parsed(name);
        for k in ["disable-settings", "disable-ab", "disable-account", "disable-installation"] {
            assert_eq!(v[k], "Y", "{name}: {k}");
        }
        assert_eq!(v["override-settings"]["enable-check-update"], "N", "{name}");
        assert_eq!(v["override-settings"]["allow-auto-update"], "N", "{name}");
        assert_eq!(v["override-settings"]["enable-clipboard"], "Y", "{name}");
        assert_eq!(v["override-settings"]["enable-file-transfer"], "Y", "{name}");
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
    assert_eq!(custom_client_source(Some(json)), CustomClientSource::Embedded(json));
    assert_eq!(custom_client_source(None), CustomClientSource::File);
}

#[test]
fn applying_destek_sets_app_name_and_hard_settings() {
    // Only test that touches the global config statics.
    crate::common::apply_custom_client_json(variant_json("destek").unwrap().as_bytes());
    assert_eq!(hbb_common::config::APP_NAME.read().unwrap().as_str(), "Atlas Destek");
    let hard = hbb_common::config::HARD_SETTINGS.read().unwrap();
    assert_eq!(hard.get("conn-type").map(String::as_str), Some("incoming"));
    assert_eq!(hard.get("disable-settings").map(String::as_str), Some("Y"));
}
```

- [ ] **Step 2: Çalıştır, başarısız olduğunu gör.** Yerelde Rust yoksa push edip CI'daki `cargo test` adımında gör. Beklenen: `variant_json` / `apply_custom_client_json` tanımsız → derleme hatası.

- [ ] **Step 3: Uygula.** JSON'ları yaz; `src/atlas.rs`'de `variant_json` (`include_str!("../res/atlas/destek.json")` vb.) ve `embedded_config` (`option_env!("ATLAS_VARIANT")` üzerinden); `common.rs`'de `apply_custom_client_json`'ı ayır, `read_custom_client` doğrulamadan sonra onu çağırsın; `load_custom_client()`'ın **ilk satırı**: `if let CustomClientSource::Embedded(json) = crate::atlas::custom_client_source(crate::atlas::embedded_config()) { apply_custom_client_json(json.as_bytes()); return; }` (debug `./custom.txt` dalından da önce).

- [ ] **Step 4: İş akışını iki varyanta aç.** Matris: `variant: [destek, baglanti]` + `include` ile `pack_exe: AtlasDestek.exe` / `AtlasBaglanti.exe`. İş seviyesinde `env: ATLAS_VARIANT: ${{ matrix.variant }}` (hem `build.py` hem `cargo test` görsün). `build.py`'den sonra `rustdesk/rustdesk.exe` → `rustdesk/${{ matrix.pack_exe }}` olarak yeniden adlandır; `PACK_EXE` = `matrix.pack_exe`. Paketleyici çıktısını `${{ matrix.pack_exe }}` adıyla yükle. Paketlemeden sonra doğrulama adımı: `rustdesk/` klasöründe `rustdesk.exe` **olmamalı** ve `${{ matrix.pack_exe }}` olmalı, yoksa iş kırmızı (Review Focus 1).

- [ ] **Step 5: Push, CI'da testlerin geçtiğini gör.** Beklenen: iki iş yeşil, `cargo test` içinde 6 `atlas::tests` testi PASS, iki artifact.

- [ ] **Step 6: Elle doğrula (temiz Windows, Türkçe yerel ayar).**
  - `AtlasDestek.exe` → ana ekranda ID + şifre, birkaç saniyede "Hazır"; Ayarlar ve "Bağlan" kutusu yok; `%LOCALAPPDATA%\atlasdestek` açma klasörü oluşur.
  - `AtlasBaglanti.exe` → bağlan kutusu var, kendi ID/şifre paneli yok; `%LOCALAPPDATA%\atlasbaglanti` oluşur; `%APPDATA%` altında `Atlas Bağlantı` ayar klasörü oluşur ve bağlantı kurulur (IPC çalışıyor demektir). Bağlantı kurulamıyor ve log'da pipe/klasör hatası varsa: `app-name`'i `AtlasBaglanti` yap, görünen adı Task 3'te Flutter metniyle ver (Review Focus 3).
  - Teknisyen exe'siyle müşteri exe'sine bağlan: görüntü, klavye/fare çalışır.

- [ ] **Step 7: Commit ve push.**

```powershell
git add res/atlas src/atlas.rs src/lib.rs src/common.rs .github/workflows/atlas-windows.yml
git commit -m "Embed Atlas server settings and locks at build time for two variants" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 3: Marka

**Files:**
- Create: `res/atlas/app-icon-512.png` (kaynak: `D:\HERD\atlantik-crm\public\images\app-icon-512.png`)
- Create: `flutter/assets/logo.png` (ana ekran logosu, `common.dart:3797` `_kDefaultLogoAsset`), `flutter/assets/icon.png` (`loadIcon`, `common.dart:3865`)
- Modify (üzerine yaz): `res/icon.ico`, `res/tray-icon.ico`, `res/icon.png`, `res/32x32.png`, `res/64x64.png`, `res/128x128.png`, `res/128x128@2x.png`, `flutter/windows/runner/resources/app_icon.ico`
- Modify: `flutter/windows/runner/Runner.rc:92-98`, `libs/portable/Cargo.toml:29-34`
- Modify: görünür sabit "RustDesk" metni bulunan Flutter dosyaları (tarama sonucu; aşağıda)

**Interfaces:**
- Consumes: Task 2 varyant adları (`get_app_name()` zaten varyant adını döndürür).
- Produces: iş akışında meta veri doğrulama adımı.

- [ ] **Step 1: Başarısız doğrulama adımını ekle.** `atlas-windows.yml`'de paketlemeden sonra PowerShell adımı:

```powershell
$v = (Get-Item ".\${{ matrix.pack_exe }}").VersionInfo
if ($v.ProductName -ne 'Atlas') { throw "ProductName: $($v.ProductName)" }
if ($v.CompanyName -ne 'Atlantik Yazılım') { throw "CompanyName: $($v.CompanyName)" }
if ($v.LegalCopyright -notlike '*RustDesk tabanlıdır (AGPL-3.0)*') { throw "LegalCopyright: $($v.LegalCopyright)" }
```
Push → beklenen: bu adım `ProductName: RustDesk` ile kırmızı.

- [ ] **Step 2: İkonları üret.** ImageMagick ile `res/atlas/app-icon-512.png`'den: `.ico` dosyaları 16/32/48/128/256 katmanlı (`res/gen_icon.sh` mantığı), PNG'ler adlarındaki boyutta, `flutter/assets/logo.png` 512, `flutter/assets/icon.png` 256. Beyaz zemin korunur.

- [ ] **Step 3: Meta veriyi yaz.** `Runner.rc` ve `libs/portable/Cargo.toml` `[package.metadata.winres]` değerleri Global Constraints'teki değerlerle; `InternalName`/`OriginalFilename` `atlas`/`Atlas.exe`. `.rc` dosyası UTF-8 değilse `ı` için kod sayfasını (`#pragma code_page(65001)`) ekle; `Translation` `0x041F, 1200` (Türkçe, Unicode).

- [ ] **Step 4: Görünür metin taraması.** `flutter/lib` ve `src/lang/tr.rs` içinde ana pencere, tepsi menüsü, bağlantı kabul (CM) penceresi ve uzak oturum araç çubuğunda görünen sabit `RustDesk` geçişlerini listele; `get_app_name()`/`bind.mainGetAppNameSync()` kullanmayanları ona çevir. Telif bildirimleri, lisans metinleri, log satırları ve çeviri dosyalarının geri kalanı dokunulmaz. Değişen dosya sayısı 10'u geçerse dur ve listeyi kullanıcıya getir (upstream sapması anti-hedefi).

- [ ] **Step 5: Push, CI yeşil.** Meta veri adımı geçer.

- [ ] **Step 6: Elle doğrula.** Exe ikonu, görev çubuğu, tepsi ve ana ekranda Atlantik logosu; pencere başlıkları "Atlas Destek"/"Atlas Bağlantı"; dosya Özellikler → Ayrıntılar'da Atlas/Atlantik Yazılım.

- [ ] **Step 7: Commit ve push.**

```powershell
git add res flutter/assets flutter/windows/runner libs/portable/Cargo.toml flutter/lib src/lang .github/workflows/atlas-windows.yml
git commit -m "Brand the Atlas builds with the Atlantik logo and product metadata" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 4: Sürüm ve elle kabul

**Files:**
- Modify: `.github/workflows/atlas-windows.yml` (release işi)

**Interfaces:**
- Consumes: Task 2–3 artifact'ları `AtlasDestek.exe`, `AtlasBaglanti.exe`.
- Produces: GitHub Release `atlas-v1.5.3-atlas.1`; Task 5 bu Release'teki `AtlasDestek.exe`'yi kullanır.

- [ ] **Step 1: Release işini ekle.** `needs: build-windows`, yalnızca `atlas-v*` etiketinde; iki exe'yi indirir, `SHA256SUMS.txt` üretir, `softprops/action-gh-release` (upstream'in sabitlediği SHA ile) ile yayınlar. Release gövdesi Türkçe: iki exe'nin ne olduğu, "imzasızdır, SmartScreen'de Daha fazla bilgi → Yine de çalıştır", "RustDesk 1.5.3 tabanlıdır, AGPL-3.0; kaynak kodu bu depodadır" + `atlas` dalı bağlantısı.

- [ ] **Step 2: Etiketle.**

```powershell
git add .github/workflows/atlas-windows.yml
git commit -m "Publish Atlas builds as a GitHub release on atlas tags" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git tag atlas-v1.5.3-atlas.1
git push origin atlas atlas-v1.5.3-atlas.1
```
Beklenen: Release'te iki exe + `SHA256SUMS.txt`.

- [ ] **Step 3: Elle kabul — spec'teki listenin tamamı** (temiz Windows 10 ve 11; kullanıcıyla birlikte, sonuçlar Release notuna değil `docs/atlas/kabul-1.5.3-atlas.1.md`'ye):
  1. Destek: logo, ID, şifre, "Hazır"; Ayarlar/bağlan kutusu yok.
  2. Bağlantı exe'siyle bağlan; görüntü, klavye/fare.
  3. Pano iki yönde (metin); dosya aktarımı iki yönde.
  3a. Şifreyle bağlan → oturum bitince şifre değişmiş; şifresiz bağlan → müşteride "Kabul Et" → bağlantı; kalıcı şifre ayarlanacak yer yok.
  3b. Teknisyen yetki yükseltir → müşteri UAC "Evet" → teknisyen bir UAC penceresini görür ve tıklar.
  4. Relay zorlanmış ağda (teknisyen telefonun hücresel bağlantısında) 2–3 tekrar.
  5. Tam ekran video 2 dk; aynı ağda resmi RustDesk 1.5.3 ile gözle karşılaştırma.
  6. Resmi RustDesk + iki Atlas aynı makinede birlikte açık, çakışma yok.
  7. Ağ izleme (Resource Monitor / `Get-NetTCPConnection` + DNS önbelleği) — `*.rustdesk.com` yok.
  8. Anahtarsız resmi istemci sunucuya bağlanamaz.
  Herhangi biri başarısızsa ilgili task'a dön, `atlas.2` ile yeniden etiketle.

- [ ] **Step 4: Kabul notunu commit et.**

```powershell
git add docs/atlas/kabul-1.5.3-atlas.1.md
git commit -m "Record acceptance results for 1.5.3-atlas.1" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 5: Sitede yeni exe (atlantik-website)

Yalnızca Task 4 kabulü geçtikten sonra.

**Files:**
- Modify: `D:\HERD\atlantik-website\app\Http\Controllers\RemoteSupportController.php` (`DOWNLOAD_NAME`, `content()` cta notu, yeni `SOURCE_URL`)
- Modify: `D:\HERD\atlantik-website\resources\views\pages\uzak-destek.blade.php` (kaynak kod satırı)
- Test: `D:\HERD\atlantik-website\tests\Feature\RemoteSupportTest.php`

**Interfaces:**
- Consumes: Release `atlas-v1.5.3-atlas.1` içindeki `AtlasDestek.exe`.
- Produces: `RemoteSupportController::DOWNLOAD_NAME = 'AtlasDestek.exe'`, `RemoteSupportController::SOURCE_URL = 'https://github.com/fatihgormus81/atlas-rustdesk/tree/atlas'`. `STORAGE_PATH` aynı formülle (`downloads/AtlasDestek.exe`). `CONFIG_STRING` ve Mac/Linux `PLATFORMS` değişmez.

- [ ] **Step 1: Testleri güncelle (önce kırmızı).**
  - `downloads the windows client …` testi: Content-Disposition `attachment; filename=AtlasDestek.exe` (tırnak opsiyonel, platform testindeki regex kalıbı) ve içerik; dosya adındaki `host=`/`key=` beklentileri kaldırılır.
  - `encodes the same server …` testi: son satırdaki `DOWNLOAD_NAME` karşılaştırması kaldırılır; JSON beklentisi aynen kalır.
  - Yeni test `it('links to the source code of the windows client', …)`: sayfa `href="https://github.com/fatihgormus81/atlas-rustdesk/tree/atlas"` ve `Kaynak Kodu` metnini içerir.

- [ ] **Step 2: Çalıştır, kırmızı.** `php artisan test --compact tests/Feature/RemoteSupportTest.php` → yeni test ve indirme testi FAIL.

- [ ] **Step 3: Uygula.** Sabitler; cta notu `Windows 10 ve 11 · Atlas Destek`; Windows bölümünün altına tek satır: `Atlas, açık kaynak RustDesk tabanlıdır (AGPL-3.0).` + `<a href="{{ $sourceUrl }}">Kaynak Kodu</a>` (`show()` `sourceUrl` geçirir). Sayfa metninde "RustDesk" geçen yerleri gözden geçir; marka adı "Atlas Destek" olur, tabanlık bilgisi bu satırda kalır.

- [ ] **Step 4: Yeşil + biçim.** `php artisan test --compact tests/Feature/RemoteSupportTest.php` → PASS; `vendor/bin/pint --dirty --format agent`.

- [ ] **Step 5: Commit** (push yalnızca kullanıcı isteyince).

```powershell
git -C D:\HERD\atlantik-website add app/Http/Controllers/RemoteSupportController.php resources/views/pages/uzak-destek.blade.php tests/Feature/RemoteSupportTest.php
git -C D:\HERD\atlantik-website commit -m "Serve the branded Atlas Destek client on the remote support page with a source code link" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 6: Kullanıcıya yayın notu.** Push sonrası sunucuya kopyalanacak dosya: `storage/app/private/downloads/AtlasDestek.exe` (Release'ten). Eski uzun adlı exe silinebilir. Proje hafızasındaki dağıtım notu (`rustdesk-server-lxc.md`) yeni dosya adıyla güncellenir.
