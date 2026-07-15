# aenyrathia

Git-backed wiki for the Aenyrathia setting.

## Development

Run checks:

```sh
just nice
just test
```

Run locally with SQLite:

```sh
DATABASE_URL=sqlite://data/aenyrathia.sqlite3 cargo run
```

Build the Nix package:

```sh
nix build .#aenyrathia
```

For local HTTP development, leave `COOKIE_SECURE` unset so login cookies work on `http://127.0.0.1:8080`.

## Deployment notes

For HTTPS deployments, set:

```sh
COOKIE_SECURE=true
```

This marks auth/CSRF cookies as Secure so browsers only send them over HTTPS.

Use an explicit deploy database path, for example:

```sh
DATABASE_URL=sqlite:///var/lib/aenyrathia/aenyrathia.sqlite3
```
