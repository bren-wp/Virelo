# Virelo

**Virelo** je desktop poslovni sustav za organizaciju podataka firme, klijenata, kontakt-osoba, projekata, zadataka, bilješki, dokumenata, aktivnosti i financijske evidencije.

## Virelo 1.2

Virelo objedinjuje:

- podatke firme: OIB / PIB / DIČ, MBO / MBS / IČO, adresa, kontakti, IBAN, SWIFT/BIC i banka
- klijente s poslovnim i poreznim podacima
- kontakt-osobe svakog klijenta s funkcijom, e-mailom, telefonom i bilješkama
- projekte sa statusima, prioritetima, rokovima i vrijednostima
- zadatke povezane s klijentima i projektima
- Markdown bilješke i oznake
- dokumente i ugovore povezane s poslovnim zapisima
- kronologiju poziva, sastanaka, e-mailova i drugih aktivnosti
- ponude, račune, troškove i druge financijske zapise
- globalnu pretragu kroz sve glavne module
- sigurnosnu kopiju baze
- izvoz u JSON, CSV, Markdown i HTML

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

## Izvoz podataka

Virelo podržava više otvorenih formata:

- **JSON** — potpuni strukturirani izvoz
- **CSV** — tablični izvoz za Excel, Numbers i druge alate
- **Markdown** — čitljiva tekstualna arhiva
- **HTML** — samostalni izvještaj koji se otvara u pregledniku
- **DB sigurnosna kopija** — cjelovita kopija baze

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
