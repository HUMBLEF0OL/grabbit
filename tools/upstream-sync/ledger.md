# Upstream ledger

The original MDM's commits (anefzaoui/misterdownloadmanager, `upstream/main`) this fork decided
on without picking them. Picks need no row: their `(cherry picked from commit …)` line in this
fork's history is the record, and a port's `Upstream-Port: <sha>` trailer is too (a `ported` row
here adds the PR and a note). `tools/upstream-sync/sync.py record` writes the rows; one per
upstream commit, a later decision replacing the earlier one.

- **skipped**: not wanted here (a removed platform or language, a change this fork doesn't want)
- **covered**: the fork already fixed it its own way; the ref says where
- **deferred**: looked at, not now (it shows again in `status`); the note says what it waits for
- **ported**: written again by hand in the fork's code; the ref is the PR

| upstream | status | fork ref | upstream subject | note |
|---|---|---|---|---|
