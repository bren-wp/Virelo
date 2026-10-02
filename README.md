# Virelo

**Virelo** je local-first poslovni workspace za Windows, macOS i Linux. Podaci ostaju na korisničkom uređaju i aplikacija ne zahtijeva obavezni račun, cloud, pretplatu ni telemetriju.

## Virelo 1.0

Virelo objedinjuje:

- podatke firme: OIB / PIB / DIČ, MBO / MBS / IČO, adresa, kontakti, IBAN, SWIFT/BIC i banka
- klijente i kontakte
- projekte s rokovima, statusima, prioritetima i vrijednostima
- zadatke povezane s klijentima i projektima
- Markdown bilješke i tagove
- lokalne dokumente
- kronologiju aktivnosti: pozivi, sastanci, e-mailovi, ugovori i bilješke
- lokalnu evidenciju ponuda, računa i troškova
- globalnu pretragu cijelog workspacea
- SQLite backup
- JSON izvoz workspacea

## Privatnost

Virelo je local-first:

- nema obavezne registracije
- nema obaveznog clouda
- nema telemetrije
- nema zaključavanja podataka u proprietarni servis
- poslovni zapisi spremaju se u lokalnu SQLite bazu
- uvezeni dokumenti kopiraju se u Virelo application-data direktorij

## Tehnologije

- Tauri 2
- Rust
- Svelte 5 / SvelteKit
- TypeScript
- SQLite

## Windows buildovi

GitHub Actions proizvodi i provjerava:

- `Virelo-Setup.exe` — NSIS instalacijski paket
- `Virelo-Portable.exe` — samostalni Windows executable

Windows pipeline provjerava da oba artefakta postoje i imaju valjano PE zaglavlje te pokreće Portable EXE kao launch smoke test.

Na uspješnom buildu grane `main` buildovi se objavljuju u GitHub Release **v1.0.0**.

## Lokalni razvoj

Potrebni su Node.js 22+, Rust stable i platform-specific Tauri dependencies.

```bash
npm install
npm run check
npm run build
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
```

Pokretanje desktop aplikacije:

```bash
npm --workspace @virelo/desktop run tauri -- dev
```

Produkcijski Tauri build:

```bash
npm --workspace @virelo/desktop run tauri -- build
```

## Sigurnost podataka

SQLite koristi foreign keys i WAL način rada. Brisanje upravljanih dokumenata ograničeno je na Virelo documents direktorij. Backup i JSON izvoz korisnik pokreće ručno i bira vlastitu lokaciju.

## Licenca

MIT
