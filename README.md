<p align="center">
  <img src="docs/brand/virelo-logo.svg" width="112" height="112" alt="Virelo logo">
</p>

<h1 align="center">Virelo</h1>

<p align="center">
  <strong>Poslovni workspace koji ostaje na vašem računalu.</strong><br>
  Klijenti, kontakti, ugovori, projekti, zadaci, dokumenti, financije i sigurnosne kopije — u jednoj desktop aplikaciji za Windows, macOS i Linux.
</p>

<p align="center">
  <a href="https://github.com/bren-wp/Virelo/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/bren-wp/Virelo?display_name=tag&style=for-the-badge"></a>
  <a href="https://github.com/bren-wp/Virelo/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/bren-wp/Virelo/ci.yml?branch=main&label=CI&style=for-the-badge"></a>
  <a href="LICENSE"><img alt="MIT license" src="https://img.shields.io/badge/license-MIT-5565e8?style=for-the-badge"></a>
</p>

<p align="center">
  🪟 <strong>Windows</strong> &nbsp;·&nbsp; 🍎 <strong>macOS Apple Silicon + Intel</strong> &nbsp;·&nbsp; 🐧 <strong>Linux</strong>
</p>

---

## ✨ Poslovanje bez nepotrebnog clouda

Virelo je desktop poslovni sustav za male tvrtke, obrte, samostalne profesionalce i timove koji žele organizirati poslovne podatke bez obaveznog korisničkog računa i bez obaveznog slanja podataka na vanjski servis.

Podaci se spremaju lokalno u SQLite bazu na uređaju. Dokumenti koje uvezete čuvaju se u lokalnoj Virelo arhivi, a cijeli workspace možete izvesti u standardni ZIP zajedno s bazom i dokumentima.

### Što dobivate

| | Modul | Što radi |
|---|---|---|
| 🏢 | **Moja firma** | OIB / PIB / DIČ, MBO / MBS / IČO, adresa, kontaktni i bankovni podaci |
| 👥 | **Klijenti** | Poslovni i porezni podaci, statusi, kontakti i bilješke |
| 👤 | **Kontakti** | Kontakt-osobe povezane s klijentima, funkcija, e-mail i telefon |
| 🏦 | **Bankovni računi** | Više računa, IBAN, SWIFT/BIC, banka, valuta i zadani račun |
| 📑 | **Ugovori** | Klijenti, projekti, dokumenti, status, rok i vrijednost |
| 💼 | **Projekti** | Status, prioritet, rok, vrijednost i povezani zapisi |
| ✅ | **Zadaci** | Klijenti, projekti, statusi, prioriteti i rokovi |
| 📝 | **Bilješke** | Markdown sadržaj, oznake i povezivanje |
| 🗂️ | **Dokumenti** | Lokalna arhiva, kategorije, oznake, opis i veze |
| 📞 | **Aktivnosti** | Pozivi, sastanci, e-mailovi i poslovna kronologija |
| 💶 | **Financije** | Ponude, računi, troškovi, statusi, rokovi i iznosi |
| 🔎 | **Globalna pretraga** | Pretraga kroz glavne poslovne module |
| 🛡️ | **Backup i restore** | ZIP arhiva, SQLite backup, validacija i rollback |

---

## ⬇️ Preuzimanje

Najnovije stabilno izdanje uvijek je dostupno na stranici **Releases**.

| Platforma | Paket | Preuzimanje |
|---|---|---|
| 🪟 Windows | Setup | [Virelo-Setup.exe](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo-Setup.exe) |
| 🪟 Windows | Portable | [Virelo-Portable.exe](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo-Portable.exe) |
| 🍎 macOS Apple Silicon | DMG | [Virelo-macOS-arm64.dmg](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo-macOS-arm64.dmg) |
| 🍎 macOS Apple Silicon | App ZIP | [Virelo-macOS-arm64-app.zip](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo-macOS-arm64-app.zip) |
| 🍎 macOS Intel | DMG | [Virelo-macOS-x64.dmg](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo-macOS-x64.dmg) |
| 🍎 macOS Intel | App ZIP | [Virelo-macOS-x64-app.zip](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo-macOS-x64-app.zip) |
| 🐧 Linux | AppImage | [Virelo.AppImage](https://github.com/bren-wp/Virelo/releases/latest/download/Virelo.AppImage) |
| 🐧 Debian / Ubuntu | DEB | [virelo.deb](https://github.com/bren-wp/Virelo/releases/latest/download/virelo.deb) |

> **macOS potpisivanje:** CI izrađuje normalan .app i .dmg za Apple Silicon i Intel. Kada su Apple signing/notarization vjerodajnice konfigurirane kroz GitHub Secrets, workflow koristi potpisani build; bez certifikata build ostaje unsigned i macOS može prikazati Gatekeeper upozorenje.

---

## 🖥️ Jedna aplikacija, tri desktop platforme

~~~mermaid
flowchart LR
    V["Virelo workspace"] --> W["🪟 Windows<br/>Setup + Portable"]
    V --> M["🍎 macOS<br/>Apple Silicon + Intel"]
    V --> L["🐧 Linux<br/>AppImage + DEB"]
    V --> D["SQLite baza"]
    V --> F["Lokalni dokumenti"]
    V --> B["ZIP backup + restore"]
    D --> E["JSON · CSV · Markdown<br/>HTML · YAML · XML"]
    F --> B
~~~

Ista poslovna funkcionalnost koristi se na sve tri platforme. Virelo prozor je normalno promjenjive veličine, ne pokreće se prisilno preko cijelog zaslona i podržava svijetlu/tamnu temu prema postavci operacijskog sustava.

U **Sigurnosne kopije → O aplikaciji** Virelo prikazuje verziju, operacijski sustav, arhitekturu i stvarnu lokalnu mapu podataka te je može otvoriti izravno iz aplikacije.

---

## 🔐 Local-first i oporavak podataka

Virelo ne zahtijeva obaveznu prijavu niti obavezni cloud servis.

### Potpuna Virelo ZIP arhiva

ZIP arhiva sadrži SQLite bazu, čitljivi strukturirani JSON i sve dokumente spremljene u Virelo arhivu.

Prije povrata Virelo provjerava format arhive, veličine zapisa, integritet SQLite baze, obavezne tablice, strane ključeve i prisutnost dokumenata. Putanje iz ZIP-a ne smiju izaći iz staging direktorija, a ako primjena povrata ne uspije, aktivira se rollback prethodne baze i dokumenata.

### Otvoreni izvozni formati

- **JSON** — potpuni strukturirani izvoz
- **CSV** — tablični izvoz sa zaštitom od spreadsheet formula injectiona
- **Markdown** — čitljiva tekstualna arhiva
- **HTML** — samostalni izvještaj za preglednik
- **YAML** — čitljivi strukturirani format
- **XML** — standardni razmjenski format
- **SQLite DB** — lokalna sigurnosna kopija baze
- **ZIP** — potpuni workspace s dokumentima

---

## 🎯 Dizajn za svakodnevni rad

- responzivna bočna navigacija za manje prozore
- svijetla i tamna tema prema OS-u
- globalna pretraga s tipkovničkim prečacem **Ctrl/Cmd + K**
- filtriranje projekata, zadataka, financija i dokumentne arhive
- potvrde prije destruktivnih radnji
- uređivanje i povezivanje poslovnih entiteta u jednom workspaceu
- otvaranje lokalnih dokumenata u zadanoj OS aplikaciji
- otvaranje lokalne Virelo mape podataka iz aplikacije

---

## ✅ Produkcijski QA

Svaki PR prema main grani mora proći frontend i Rust provjere te produkcijske buildove za sve platforme.

### Zajedničke provjere

- npm ci
- Svelte / TypeScript provjera
- production frontend build
- cargo fmt --check
- cargo check --locked
- cargo test --locked
- cargo clippy --locked --all-targets -- -D warnings
- provjera sinkronizirane verzije kroz npm, Tauri, Cargo i lockfileove

### 🪟 Windows

- stvarni NSIS Setup build
- Portable EXE
- PE header provjera
- Windows GUI subsystem provjera — bez konzolnog prozora
- Portable launch smoke test
- tiha Setup instalacija s /S
- launch smoke test stvarno instalirane aplikacije

### 🍎 macOS

- Apple Silicon arm64 .app + .dmg
- Intel x64 .app + .dmg
- bundle identifier i version provjera
- executable provjera
- DMG integrity provjera
- stvarni .app launch smoke test
- opcionalni Apple signing/notarization put kada su vjerodajnice dostupne

### 🐧 Linux

- AppImage
- Debian/Ubuntu .deb
- DEB metadata/integrity provjera
- AppImage launch smoke test pod virtualnim displayem
- stvarna instalacija .deb paketa
- launch smoke test instalirane Linux aplikacije

---

## 🧱 Tehnologije

| Sloj | Tehnologija |
|---|---|
| Desktop runtime | **Tauri 2** |
| Backend | **Rust** |
| UI | **Svelte 5 / SvelteKit** |
| Jezik sučelja | **TypeScript** |
| Lokalna baza | **SQLite / rusqlite** |
| Desktop dijalozi | **Tauri Dialog plugin** |
| Otvaranje datoteka | **Tauri Opener plugin** |
| Buildovi | **GitHub Actions** |

---

## 🛠️ Razvoj

Projekt koristi npm lockfile i Cargo lockfile kako bi CI i lokalni buildovi koristili reproducibilne dependency verzije.

<pre><code>npm ci
node scripts/verify-version.mjs
npm run check
npm run build

cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo test --locked --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo clippy --locked --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets -- -D warnings</code></pre>

Pokretanje development aplikacije:

<pre><code>npm run tauri -- dev</code></pre>

Produkcijski build na lokalnoj platformi:

<pre><code>npm run tauri -- build</code></pre>

---

## 📦 Izdavanja

Produkcijski release nastaje nakon uspješnog platformskog builda. Windows workflow kreira verzionirani GitHub Release, a macOS i Linux workflowi na isti tag dodaju provjerene platform-specific artefakte.

👉 **[Otvori najnovije Virelo izdanje](https://github.com/bren-wp/Virelo/releases/latest)**

---

## 📄 Licenca

Virelo je objavljen pod **MIT licencom**. Pogledajte [LICENSE](LICENSE).

<p align="center">
  <img src="docs/brand/virelo-logo.svg" width="42" height="42" alt="">
  <br>
  <strong>Virelo</strong><br>
  <sub>Vaši poslovni podaci. Vaš uređaj. Vaš workspace.</sub>
</p>
