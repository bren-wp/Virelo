# Virelo Security

Virelo je desktop aplikacija u kojoj poslovni podaci i uvezeni dokumenti ostaju na korisničkom uređaju osim ako ih korisnik sam ne premjesti, izveze ili sinkronizira drugim alatom.

## Sigurnosna načela

- bez obaveznog clouda i bez obavezne prijave
- bez ugrađene telemetrije
- SQLite foreign keys, WAL način rada i provjera integriteta sigurnosnih kopija
- dokumenti se kopiraju u Virelo application-data direktorij
- brisanje upravljanih dokumenata ograničeno je na Virelo documents direktorij
- povrat ZIP arhive koristi staging, provjeru putanja i rollback ako primjena ne uspije
- CSV izvoz neutralizira vrijednosti koje tablični alati mogu protumačiti kao formule
- desktop CSP ne dopušta inline stilove

## Zaštita podataka na uređaju

Virelo trenutno ne enkriptira SQLite bazu vlastitim formatom i ne pohranjuje ključeve u OS keychain. Za podatke na izgubljenom ili ukradenom uređaju preporučuje se uključena enkripcija cijelog diska i zaštićen korisnički račun operacijskog sustava (npr. BitLocker, FileVault ili odgovarajuća Linux enkripcija).

Sigurnosne kopije i izvozi također mogu sadržavati povjerljive poslovne podatke te ih treba spremati na zaštićenu lokaciju.
