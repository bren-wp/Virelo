# Virelo Security

Virelo je local-first desktop aplikacija. Poslovni podaci i uvezeni dokumenti ostaju na korisničkom uređaju osim ako ih korisnik sam ne premjesti ili sinkronizira drugim alatom.

## Sigurnosna načela

- bez obaveznog clouda
- bez telemetrije
- SQLite foreign keys
- WAL način rada
- dokumenti se kopiraju u Virelo application-data direktorij
- brisanje upravljanih dokumenata ograničeno je na Virelo documents direktorij

Enkripcija baze i integracija s OS keychainom planirane su prije stabilnog 1.0 izdanja.
