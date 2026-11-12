# Upstream ledger

What this fork decided about each of the original MDM's commits (anefzaoui/misterdownloadmanager,
`upstream/main`) since the fork point. `tools/upstream-sync/sync.py` writes the rows (`pick`
and `record`): one per upstream commit, a later decision replacing the earlier one. Edit by hand
only to fix a mistake; `sync.py doctor` reports a bad or duplicated row.

- **picked**: taken as it is (minus what the fork removed); the ref is the fork's commit, whose
  `Upstream-Pick: <sha>` trailer must reach `dev` (a squash merge drops it: `status` then shows
  `picked?`)
- **ported**: written again by hand in the fork's code; the ref is the PR, whose commit carries
  `Upstream-Port: <sha>` (until it merges, `status` shows `ported?`)
- **skipped**: not wanted here (a removed platform or language, an upstream feature)
- **covered**: the fork already fixed it its own way; the ref says where
- **deferred**: looked at, not now (it shows again); the note says what it waits for

| upstream | status | date | ref | upstream subject | note |
|---|---|---|---|---|---|
