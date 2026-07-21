# Backup tar/gz creation, extraction & restore (path traversal)

3 confirmed (1 medium, 2 low as unverified), 0 refuted.

## Confirmed findings

### MEDIUM [correctness]: Failed backup restore leaves the global DB connection pool permanently closed (app-wide outage until restart)

- **Attack/trigger:** An authenticated admin (session cookie, Role::Admin) POSTs /api/backups/{filename}/restore for a filename that passes is_safe_backup_filename but whose archive is corrupt/truncated, missing the database.db entry, or unreadable by GzDecoder/Archive. backup::restore_backup returns Err, the handler returns InternalError, and the closed pool is never replaced. Transient sqlx reconnect failures in the retry loop cause the same permanent-closed-pool state even for a valid archive.
- **Location:** `server/src/routes.rs:438-449, 526-530`
- **What happens:** restore_backup() unconditionally closes the live shared pool (current_pool.close() at line 440) BEFORE doing anything reversible, then calls backup::restore_backup(). If the restore returns Err (lines 446-449 propagate with `?`) OR all three pool-reconnect retries fail (lines 526-529 return DatabaseError), the handler returns early and db.replace_pool(new_pool) (line 512) is never reached. The Database's pool remains the closed one, so every subsequent request that calls db.pool() gets a closed pool and fails.
- **Why:** The teardown (pool close) is eager and up front, but the only path that restores service (replace_pool) is on the success branch; any failure between close() and replace_pool() leaves a closed pool with no recovery, turning a failed admin operation into an app-wide DoS.
- **Fix sketch:** Do not tear down the live pool until a validated new DB is staged: validate/open the restored DB into a fresh pool first, then close+swap. On ANY error path, re-establish a working pool (reconnect + replace_pool) before returning, or keep the original pool alive until replacement succeeds. Also validate the archive is openable and contains database.db before current_pool.close().

### LOW [correctness]: Backups are written non-atomically to their final path, leaving truncated archives that surface in the UI and can be selected for restore (unverified)

- **Attack/trigger:** Server crash / OOM / disk-full during a scheduled Auto backup (main.rs:134) or an admin manual backup (POST /api/backups) leaves a partial archive. An admin restoring that leftover file then triggers the permanent-closed-pool outage described in the restore finding.
- **Location:** `server/src/backup.rs:94`
- **What happens:** create_backup opens fs::File::create(&backup_path) directly on the final swolemate_*.tar.gz path and streams the tar/gzip in place (lines 94-196). If the process is killed or an IO/DB error occurs after the file is created but before archive.finish(), a truncated/partial .tar.gz remains under the real backup name. list_backups skips it only if metadata.json is unreadable; a partial archive that flushed metadata first still occupies a valid, selectable backup name.
- **Why:** Direct-to-final-path writes for archive artifacts are not crash-safe; the standard pattern is write-to-temp then atomic rename so only fully-formed archives ever appear under a real backup name.
- **Fix sketch:** Write to a temp path (e.g. *.tar.gz.tmp), call archive.finish() + fsync, then fs::rename() to the final backup_path; delete the temp file on error. Guarantees only complete backups are ever visible/restorable.

### LOW [security]: Restore extraction relies on an entry-name whitelist and would follow symlink/hardlink tar entries if an untrusted-archive ingress were ever added (unverified)

- **Attack/trigger:** Not reachable in the current codebase (no route or vector lets an attacker place a crafted .tar.gz into backups/). It becomes an arbitrary-file-write-on-restore primitive only if a future feature allows importing/uploading backup archives, or if backups/ becomes writable by an untrusted party.
- **Location:** `server/src/backup.rs:250-265`
- **What happens:** restore_backup unpacks only entries whose path exactly equals database.db/.db-wal/.db-shm into fixed controlled destinations (entry.unpack(&temp_new_db|wal|shm)); all other entries are ignored. However, tar::Entry::unpack honors the entry typeflag: a symlink/hardlink entry named database.db would be materialized as a symlink at temp_new_db and then fs::rename'd to db_path (line 276), after which the reopened SQLite pool writes through the symlink to an arbitrary path.
- **Why:** Defense-in-depth: safety here is contingent on the archive source being fully trusted, not on hardened extraction. Adding a backup-import feature would silently turn this into a critical vuln because the extraction never rejects non-regular-file entry types.
- **Fix sketch:** Assert entry.header().entry_type().is_file() and reject anything else; unpack raw bytes with io::copy into an O_CREAT|O_EXCL file instead of entry.unpack(). If an import feature is ever added, also enforce a decompressed-size cap on the gzip stream (flate2 GzDecoder is unbounded) and validate the archive before touching the live DB.

## Refuted (not real / already handled)

None.
