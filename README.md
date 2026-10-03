# Virelo

**Virelo** je desktop poslovni sustav za organizaciju podataka firme, klijenata, kontakt-osoba, projekata, zadataka, bilješki, dokumenata, aktivnosti i financijske evidencije.

## Virelo 1.8

Virelo objedinjuje:

- podatke firme: OIB / PIB / DIČ, MBO / MBS / IČO, adresa, kontakti, IBAN, SWIFT/BIC i banka
- klijente s poslovnim i poreznim podacima
- kontakt-osobe svakog klijenta s funkcijom, e-mailom, telefonom i bilješkama
- više bankovnih računa s IBAN-om, SWIFT/BIC-om, bankom, valutom i zadanim računom
- ugovore povezane s klijentima, projektima i dokumentima, uz status, rok i vrijednost
- projekte sa statusima, prioritetima, rokovima i vrijednostima
- zadatke povezane s klijentima i projektima
- Markdown bilješke i oznake
- dokumente i ugovore povezane s poslovnim zapisima
- kronologiju poziva, sastanaka, e-mailova i drugih aktivnosti
- ponude, račune, troškove i druge financijske zapise
- globalnu pretragu kroz sve glavne module
- potpunu ZIP arhivu s bazom i dokumentima, provjereni povrat arhive i rollback pri pogrešci
- sigurnosnu kopiju baze i provjereni povrat SQLite kopije
- izvoz u JSON, CSV, Markdown, HTML, YAML i XML

## Windows aplikacija

Windows izdanje proizvodi:

- `Virelo-Setup.exe` — instalacijska verzija
- `Virelo-Portable.exe` — prijenosna verzija

Produkcijski pipeline provjerava:

- Svelte provjeru i production frontend build
- Rust format, compile, funkcionalne testove i clippy bez upozorenja
- Windows PE zaglavlje
- Windows GUI subsystem, kako se uz aplikaciju ne bi otvarao konzolni prozor
- stvarno pokretanje Portable EXE-a kao launch smoke test

## Podržane platforme

Produkcijski buildovi održavaju jednake poslovne funkcije na podržanim desktop platformama:

- **Windows** — `Virelo-Setup.exe` i `Virelo-Portable.exe`
- **macOS Apple Silicon** — `Virelo-macOS-arm64.dmg` i `Virelo-macOS-arm64-app.zip`
- **macOS Intel** — `Virelo-macOS-x64.dmg` i `Virelo-macOS-x64-app.zip`
- **Linux** — `Virelo.AppImage` i `virelo.deb`

macOS build ostaje unsigned kada signing certifikat nije konfiguriran. Workflow je pripremljen tako da signing/notarization podaci mogu biti dodani kroz GitHub Secrets bez blokiranja unsigned builda.

## Izvoz podataka

Virelo podržava više otvorenih formata:

- **JSON** — potpuni strukturirani izvoz
- **CSV** — tablični izvoz za Excel, Numbers i druge alate
- **Markdown** — čitljiva tekstualna arhiva
- **HTML** — samostalni izvještaj koji se otvara u pregledniku
- **YAML** — čitljiv strukturirani format
- **XML** — standardni razmjenski format za druge sustave
- **ZIP arhiva** — baza, strukturirani JSON i svi spremljeni dokumenti; preporučeni format za potpuni oporavak ili prijenos na drugi uređaj
- **DB sigurnosna kopija** — cjelovita kopija baze; povrat je dostupan kada pripadajući dokumenti već postoje u lokalnoj Virelo arhivi

## Siguran povrat podataka

Povrat ZIP arhive prije izmjene aktivnih podataka provjerava format arhive, veličine zapisa, integritet SQLite baze, obavezne tablice, strane ključeve i prisutnost svih dokumenata. Putanje iz ZIP-a ne mogu izaći iz Virelo staging mape, a pri grešci se automatski vraćaju prethodna baza i dokumenti.

CSV izvoz neutralizira vrijednosti koje bi tablični programi mogli protumačiti kao formule (`=`, `+`, `-`, `@`), čime se smanjuje rizik formula injectiona pri otvaranju izvoza u Excelu, Numbersu ili sličnim alatima.

## Tehnologije

- Tauri 2
- Rust
- Svelte 5 / SvelteKit
- TypeScript
- SQLite

## Razvoj

```bash
npm install
npm run check
npm run build
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets -- -D warnings
```

## Licenca

MIT
