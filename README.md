# WG

Track chores of a shared household

## Build

### Docker

```bash
docker build -t wg .
```

### Cargo

```bash
cargo build --release
```

## Run

### Docker

```bash
docker run -p 3000:3000 -v "$(pwd)/data:/data" --rm wg
```

### Docker Compose

```yaml
services:
  wg:
    image: wg
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
```

### Binary

```bash
wg
```

## Configuration

The following environment variables can be used for configuration:

| Variable   | Default          |
| ---------- | ---------------- |
| `PORT`     | 3000             |
| `DB_FILE`  | ./data/sqlite.db |
| `RUST_LOG` | error            |

## FAQ

### How do the points and score work?

Each chore has a certain amount of points - you can define this yourself. Usually more difficult or work-intensive chores have more points. Once you finish a chore you gain that amount of points. There is also a view to see how many points your roommates have in comparison to you. You can configure your chore list to reset the points/score of all users in the following intervals: monthly, quaterly, half-yearly, yearly, never.

### Whats with the name "WG"?

WG is short for "Wohn-gemeinschaft" which is german for "shared apartment".

## License

Copyright (C) Oliver Amann

This project is licensed under the GNU Affero General Public License Version 3 (AGPL-3.0-only). Please see [LICENSE](./LICENSE) for more information.
