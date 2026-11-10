import json,re,base64,sys,os
S=sys.argv[1];REPO=sys.argv[2]
T=f"{S}/pkgs/tabler-icons-3.31.0/package/icons/outline"
src=open(f"{S}/work/iconmap.py").read()
MAP={n:t for n,t,p in re.findall(r'"([a-z-]+)": \("([a-z0-9-]+)","([a-z0-9-]+)"\)',src)}
REFINE={"tb-stop-all":"player-stop","tb-start-queue":"clock-play","tb-stop-queue":"clock-pause","tb-scheduler":"calendar-time","tb-delete-completed":"checks","ui-restore":"squares","st-queued":"clock-hour-4","tb-add-url":"link-plus","ui-to-tray":"layout-bottombar-collapse"}
changed=[k for k,v in REFINE.items() if MAP.get(k)!=v]
MAP.update(REFINE)
EXTRA={"x-check":"check","x-chevron-down":"chevron-down","x-chevron-up":"chevron-up","x-chevron-right":"chevron-right","x-search":"search","x-sidebar":"layout-sidebar-right","x-link":"link","x-download":"download","x-window":"app-window","x-external":"external-link","x-apps":"apps","x-edit":"pencil","x-refresh":"refresh","x-shield":"shield-check","x-info":"info-circle","x-plus":"plus","x-dots":"dots","x-grip":"grip-vertical","x-gauge":"gauge","x-adjust":"adjustments","x-network":"sitemap","x-key":"key","x-user":"user","x-volume":"volume","x-world":"world","x-device":"device-mobile","x-arrow":"arrow-right","x-filter":"filter","x-hash":"hash","x-copy":"copy","x-rss":"rss","x-upload":"upload","x-minus":"minus","x-pointer":"pointer","x-merge":"arrows-join","x-power":"power","x-browser":"browser","x-bug":"bug","x-keyboard":"keyboard","x-send":"send","x-gift":"gift"}
def load(n):
    v=open(f"{T}/{n}.svg",encoding="utf8").read()
    v=re.sub(r"\s+"," ",v).replace("> <","><");v=v.replace('<path stroke="none" d="M0 0h24v24H0z" fill="none"/>',"")
    v=re.sub(r' class="[^"]*"',"",v);v=re.sub(r' stroke-width="2"',"",v);v=re.sub(r' (width|height)="24"',"",v)
    return v.strip()
IC={k:load(v) for k,v in {**MAP,**EXTRA}.items()}
labels=dict(re.findall(r'\(\s*"([a-z-]+)",\s*"([^"]+)",?\s*\)',open(f"{REPO}/crates/app/src/themeicons.rs",encoding="utf8").read()))
import pal as PALMOD
css,rows=PALMOD.build()
VERDICT={
 "graphite":("Today's direction. Calm and safe, and green also reads as done, which suits a download manager. The least distinctive of the four.",False),
 "obsidian":("The boldest and most pro, now with a steel ice-blue primary instead of plain black and a warmer racing red. Red still means Stop and Error in a download manager, so errors move to amber; red progress stays the risky part.",False),
 "amethyst":("The most distinctive, with the clearest meanings: amethyst for what you chose, cyan for what is moving, red free for errors. Violet-tinted greys tie it together, and on jet black the cyan glows. In light mode it deepens to a teal-cyan to stay readable.",True),
 "seal":("The most grown-up. Blue-tinted greys under navy and seal blue feel calm and trustworthy; crimson pink, now richer, gives progress energy without reading as an error, which moves to orange.",False),
}
def card(k,P):
    L,D=P["light"],P["dark"]
    sw=lambda role,key: f'<span>{role}</span><i><b style="background:{L[key]}"></b>{L[key]}</i><i><b style="background:{D[key]}"></b>{D[key]}</i>'
    def side(T,mode):
        return (f'<div style="background:{T["surface"]};color:{T["ink"]}"><div class="row3"><span class="ck" style="background:linear-gradient(135deg,{T["acc"]},{T["out"]});color:{T["onout"]};font-size:9px">G</span><span class="ck" style="background:{T["acc"]};color:{T["onacc"]}">&#10003;</span><span>Selected</span>'
                f'<span style="margin-left:auto;background:{T["sel"]};color:{T["acct"]};padding:2px 6px;border-radius:5px">{mode}</span></div>'
                f'<div class="pb" style="background:{T["track"]}"><i style="background:linear-gradient(90deg,{T["out"]},{T["out2"]});box-shadow:0 0 10px {T["glow"]}"></i></div>'
                f'<div class="row3"><span class="bt" style="background:linear-gradient(180deg,{T["out2"]},{T["out"]});color:{T["onout"]};box-shadow:0 6px 18px -8px {T["glow"]}">Download</span><span style="color:{T["err"]}">Error</span></div></div>')
    v,rec=VERDICT[k]
    return (f'<button type="button" class="pal" data-v="{k}" aria-pressed="false"><h3>{P["name"]}</h3>{"<span class=rec>Recommended</span>" if rec else ""}'
            f'<div class="sw"><span></span><span>Light</span><span>Dark</span>{sw("Base","ink")}{sw("Primary","acc")}{sw("Outlaw","out")}{sw("Error","err")}</div>'
            f'<div class="prev">{side(L,"Light")}{side(D,"Dark")}</div><p>{v}</p></button>')
palcards="".join(card(k,P) for k,P in PALMOD.PALS.items())
logo="data:image/png;base64,"+base64.b64encode(open(f"{REPO}/website/assets/logo-64.png","rb").read()).decode()
t=open(f"{S}/work/graphite.html",encoding="utf8").read()
t=t.replace("{{TOKENS}}",css).replace("{{ICONS}}",json.dumps(IC)).replace("{{NEW}}",json.dumps(changed)).replace("{{LABELS}}",json.dumps(labels)).replace("{{CONTRAST}}",json.dumps(rows)).replace("{{PALCARDS}}",palcards).replace("{{LOGO}}",logo).replace("{{ARROW}}",IC["x-arrow"])
assert "{{" not in t
open(f"{S}/mdm-graphite.html","w",encoding="utf8").write(t)
print(len(t),len(rows))
