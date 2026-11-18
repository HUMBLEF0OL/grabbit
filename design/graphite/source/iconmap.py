# need -> (tabler outline name, phosphor name)
import json, os, sys
MAP = {
 "tb-add-url": ("link-plus","link-simple"), "tb-add-torrent": ("magnet","magnet"),
 "tb-resume": ("player-play","play"), "tb-stop": ("player-pause","pause"),
 "tb-stop-all": ("hand-stop","hand-palm"), "tb-delete": ("trash","trash"),
 "tb-delete-completed": ("clear-all","broom"), "tb-options": ("settings","gear-six"),
 "tb-scheduler": ("alarm","alarm"), "tb-start-queue": ("player-track-next","fast-forward"),
 "tb-stop-queue": ("player-stop","stop"),
 "cat-all": ("stack-2","stack"), "cat-compressed": ("file-zip","file-zip"),
 "cat-documents": ("file-text","file-text"), "cat-music": ("music","music-notes"),
 "cat-programs": ("app-window","app-window"), "cat-video": ("movie","film-strip"),
 "cat-torrents": ("magnet","magnet"), "cat-unfinished": ("progress-down","hourglass-medium"),
 "cat-finished": ("circle-check","check-circle"), "cat-queues": ("list-numbers","list-numbers"),
 "cat-folder": ("folder","folder"), "cat-folder-open": ("folder-open","folder-open"),
 "misc-emblem-ok": ("circle-check","check-circle"), "misc-up": ("arrow-up","arrow-up"),
 "misc-down": ("arrow-down","arrow-down"), "misc-remove": ("x","x"),
 "ft-generic": ("file","file"), "ft-archive": ("file-zip","file-zip"),
 "ft-audio": ("file-music","file-audio"), "ft-video": ("movie","file-video"),
 "ft-image": ("photo","file-image"), "ft-document": ("file-text","file-doc"),
 "ft-pdf": ("file-type-pdf","file-pdf"), "ft-spreadsheet": ("file-spreadsheet","file-xls"),
 "ft-presentation": ("presentation","file-ppt"), "ft-text": ("file-description","file-txt"),
 "ft-code": ("file-code","file-code"), "ft-executable": ("box","app-window"),
 "ft-disk-image": ("disc","disc"), "ft-font": ("file-typography","text-aa"),
 "ft-torrent": ("magnet","magnet"),
 "st-complete": ("circle-check","check-circle"), "st-downloading": ("circle-arrow-down","arrow-circle-down"),
 "st-paused": ("player-pause","pause-circle"), "st-queued": ("clock","clock"),
 "st-error": ("alert-circle","warning-circle"), "st-seeding": ("circle-arrow-up","arrow-circle-up"),
 "ui-minimize": ("minus","minus"), "ui-maximize": ("square","square"),
 "ui-restore": ("box-multiple","copy-simple"), "ui-close": ("x","x"),
 "ui-to-tray": ("arrow-bar-to-down","arrow-line-down"),
}
PK = sys.argv[1]
T = f"{PK}/tabler-icons-3.31.0/package/icons/outline"
P = f"{PK}/phosphor-icons-core-2.1.1/package/assets"
def ph(name, w):
    suf = "" if w=="regular" else "-"+w
    return f"{P}/{w}/{name}{suf}.svg"
missing=[]
out={"tabler":{}, "ph":{}, "phduo":{}}
for need,(t,p) in MAP.items():
    for key,path in (("tabler",f"{T}/{t}.svg"),("ph",ph(p,"regular")),("phduo",ph(p,"duotone"))):
        if not os.path.exists(path): missing.append((need,key,path)); continue
        out[key][need]=open(path,encoding="utf8").read()
print("missing:",missing)
if len(sys.argv)>2: json.dump(out,open(sys.argv[2],"w"))
