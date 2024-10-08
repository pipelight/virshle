# Running Migrator CLI

Create the required database schema (tables).
Will overwrite existing database.

```sh
sea-orm-cli migrate fresh
```

Generate entities structs.
Do not use if entities already exists.

```sh
sea-orm-cli generate entity --output-dir ./entity/src
```
