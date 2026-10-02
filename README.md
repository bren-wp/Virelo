# Virelo

**Virelo** je local-first poslovni workspace za macOS, Windows i Linux.

Cilj je objediniti podatke firme, klijente, projekte, bilješke i dokumente u brzoj desktop aplikaciji koja radi bez obaveznog clouda, računa ili pretplate.

## Tehnologije

- Tauri 2
- Rust
- Svelte 5 / SvelteKit
- SQLite
- TypeScript

## Načela

- podaci ostaju lokalno na uređaju
- nema obavezne registracije
- nema telemetrije
- standardni SQLite i obične lokalne datoteke
- Virelo identitet kroz cijeli projekt
- fokus na brzinu, privatnost i jednostavan izvoz podataka

## Trenutni moduli

- Pregled
- Moja firma
- Klijenti
- Projekti
- Bilješke
- Dokumenti
- Globalna pretraga

## Razvoj

```bash
npm install
npm run tauri -- dev
```

Provjere:

```bash
npm run check
npm run build
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
```

## Licenca

MIT
