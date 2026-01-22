# Bandman

Bandman is a small web application that allows users to manage their band.

## Postgres Db

```bash
docker run --name postgres-db \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_DB=bandman \
  -p 5432:5432 \
  -d postgres  
```
