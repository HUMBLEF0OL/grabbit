# 03b · The extra glyphs

The mockup draws 53 icons through the app's existing slots (03) and 36 more that have no slot
yet: menu-row icons, button icons, the Options page icons, the Scheduler's queue icons, banner and
tile icons. This phase gives each a slot, carried only by the Graphite set.

- Branch: `ui/03b-extra-glyphs`
- Needs: 03
- Unblocks: 05b, 06, 07 (all waves), 08: each refers to these slots by name

## The glyphs

From `design/graphite/source/gen3.py` `EXTRA` (Tabler 3.31 outline, stroke 1.75). The slot name is the
mock's name without `x-`, prefixed `gx-` so it never collides with a classic need.

| Slot | Tabler | Where the mock uses it |
| --- | --- | --- |
| `gx-link` | link | paste field, Add magnet link, address fields (video, playlist, gallery, magnet lookup), Refresh download address |
| `gx-download` | download | paste bar's Download button, first-start empty state, updates, drop target, Options Downloads page |
| `gx-search` | search | title-bar Find, Find/Find Next menu rows, Look up buttons, video helpers |
| `gx-sidebar` | layout-sidebar-right | toolbar Details toggle |
| `gx-refresh` | refresh | Synchronization queue (tree and Scheduler), Redownload |
| `gx-gauge` | gauge | Download limits (Scheduler), Speed Limiter menu rows |
| `gx-filter` | filter | filtered column head, "Filters hide all" empty state, Clear all filters |
| `gx-window` | app-window | "Show progress window", tray Open |
| `gx-external` | external-link | Open (menu, Download complete) |
| `gx-apps` | apps | Open with... |
| `gx-edit` | pencil | Move/Rename, Edit queue |
| `gx-shield` | shield-check | Verify checksum, video helpers, Options Proxy / Socks page |
| `gx-info` | info-circle | Properties, Help contents |
| `gx-plus` | plus | category +, New queue, Add category, Create new queue |
| `gx-minus` | minus | Delete from queue |
| `gx-dots` | dots | every "..." browse button |
| `gx-grip` | grip-vertical | Download complete's drag handle |
| `gx-pointer` | pointer | On Double click |
| `gx-hash` | hash | Add batch download |
| `gx-copy` | copy | Add batch download from clipboard |
| `gx-rss` | rss | RSS feeds menu row, RSS feed list |
| `gx-upload` | upload | Export |
| `gx-merge` | arrows-join | video helpers (ffmpeg) |
| `gx-power` | power | queue-done countdown |
| `gx-browser` | browser | extension offer rows |
| `gx-bug` | bug | Report a problem, crash report |
| `gx-keyboard` | keyboard | Keyboard shortcuts |
| `gx-world` | world | Home Page, Options Regional page |
| `gx-send` | send | Tell a friend |
| `gx-gift` | gift | Support MDM |
| `gx-adjust` | adjustments | Options General page |
| `gx-network` | sitemap | Options Connection page |
| `gx-device` | device-mobile | Options Remote page |
| `gx-key` | key | Options Sites Logins page |
| `gx-user` | user | Options Accounts page |
| `gx-volume` | volume | Options Sounds page |

Not slots (drawn by code today, keep drawing them): `x-check` (check mark), `x-chevron-*` (combo,
spin, submenu and tree arrows). `x-arrow` exists only on the mock page.

## Steps (paths under `crates/app/`)

1. `design/graphite/source/export_icons.py`: export the table's slots too (same recolour and stroke
   as the 53), into `design/graphite/assets/icons/{16,24}/`; extend `SOURCES.txt`.
2. `src/themeicons.rs`: a second list `EXTRAS: &[(&str, &str)]` (slot, what it stands for) beside
   `NEEDS`; `Set::from_files` accepts names from either list. A lookup for an extra in a set that lacks
   it returns `None`: callers draw no icon (classic sets never have them).
3. Carry the new files in the Graphite `builtin_files!` lists; copy them into
   `assets/themes/graphite/icons/{16,24}/`.
4. Tests: `every_built_in_icon_file_is_a_need_and_carried` accepts extras; a new test checks the
   Graphite set has every `EXTRAS` slot at 16 and 24, and that no other built-in set claims one.
5. The theme help page that lists the slots (`docs/themes.md`) gains the extras, marked "drawn only
   where the theme's layout asks for them".

## Done when

- [ ] 36 extra slots, each at 16 and 24, in the Graphite set; the icon tests pass
- [ ] No window changes yet (the phases that use them draw them); no CHANGELOG line
