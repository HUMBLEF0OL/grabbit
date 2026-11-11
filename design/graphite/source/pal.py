# Grabbit colour directions, refined. Roles: base (hue-tinted neutrals), primary (selection, checks,
# focus, links), outlaw (the one hot colour: default button, progress, the mark), with a lighter
# end (out2) for its two-tone gradient and a glow used only in dark. Errors keep their own hue.
# `press` is a pressed face (a button, a toolbar button): a step deeper than `tile` and `hover`,
# which can be equal, so pressed never looks like the pointer over it (issue #7).
PALS={
 "graphite":dict(name="Graphite green",base="Zinc",primary="Emerald",outlaw="Emerald",
  light=dict(bg="#fafafa",surface="#ffffff",field="#ffffff",line="#e4e4e7",line2="#f0f0f2",ink="#18181b",ink2="#52525b",ink3="#6b6b74",dis="#a1a1aa",ctl="#8a8a93",acc="#0c704d",acct="#0c704d",onacc="#ffffff",sel="#e6f2ec",hover="#f2f2f4",track="#e4e4e7",tile="#f1f1f3",press="#e9e9ec",ring="#0c704d40",err="#b42318",dng="#b42318",ondng="#ffffff",errSoft="#fdecea",warnSoft="#fdf3e1",okt="#0c704d",chip="#e6f2ec",chipt="#0a5c3f",out="#0c704d",out2="#0c704d",onout="#ffffff",glow="transparent"),
  dark=dict(bg="#121214",surface="#18181b",field="#121214",line="#2b2b30",line2="#202024",ink="#f4f4f5",ink2="#b4b4bc",ink3="#8e8e98",dis="#5c5c66",ctl="#71717b",acc="#3ecf8e",acct="#3ecf8e",onacc="#06281a",sel="#15302a",hover="#222227",track="#2b2b30",tile="#222227",press="#2a2a30",ring="#3ecf8e55",err="#ff8b80",dng="#d92d20",ondng="#ffffff",errSoft="#3a1b1b",warnSoft="#3a2d14",okt="#3ecf8e",chip="#15302a",chipt="#7ee2b4",out="#3ecf8e",out2="#3ecf8e",onout="#06281a",glow="transparent")),
 "obsidian":dict(name="Obsidian, ice, racing red",base="Obsidian black",primary="Ice steel",outlaw="Racing red",
  light=dict(bg="#f2f4f7",surface="#ffffff",field="#ffffff",line="#d8dee6",line2="#ebeff3",ink="#0c0e12",ink2="#434c58",ink3="#5a6472",dis="#98a1ac",ctl="#7f8996",acc="#2e4258",acct="#2e4258",onacc="#ffffff",sel="#e1e8f0",hover="#ecf0f4",track="#dbe1e8",tile="#ecf0f4",press="#e2e7ed",ring="#2e425833",err="#a14a06",dng="#8f1d1d",ondng="#ffffff",errSoft="#fdf0e1",warnSoft="#fdf3e1",okt="#15803d",chip="#e1e8f0",chipt="#2e4258",out="#cf1219",out2="#e02a22",onout="#ffffff",glow="transparent"),
  dark=dict(bg="#08090b",surface="#101216",field="#0b0c0f",line="#22262d",line2="#181b20",ink="#eaf0f5",ink2="#b2bcc7",ink3="#8b95a1",dis="#59616b",ctl="#6d7681",acc="#cfdde9",acct="#cfdde9",onacc="#08090b",sel="#1c232c",hover="#171b21",track="#23282f",tile="#171b21",press="#1f232a",ring="#cfdde940",err="#f5a524",dng="#b91c1c",ondng="#ffffff",errSoft="#33240e",warnSoft="#33270f",okt="#4ade80",chip="#1c232c",chipt="#cfdde9",out="#ff2d2d",out2="#ff5a4a",onout="#0a0b0d",glow="#ff2d2d59")),
 "amethyst":dict(name="Jet black, royal amethyst, electric cyan",base="Jet black",primary="Royal amethyst",outlaw="Electric cyan",
  light=dict(bg="#f7f6fb",surface="#ffffff",field="#ffffff",line="#e3e0ec",line2="#f1eff7",ink="#0d0b14",ink2="#4b4760",ink3="#625d78",dis="#a5a0b6",ctl="#8a849f",acc="#6527d4",acct="#6527d4",onacc="#ffffff",sel="#f0e9fe",hover="#f3f1f9",track="#e4e1ed",tile="#f2f0f8",press="#eae7f2",ring="#6527d440",err="#c62828",dng="#c62828",ondng="#ffffff",errSoft="#fdeaea",warnSoft="#fdf3e1",okt="#15803d",chip="#f0e9fe",chipt="#5317b8",out="#00788e",out2="#00829a",onout="#ffffff",glow="transparent"),
  dark=dict(bg="#09080d",surface="#121019",field="#0c0b11",line="#262233",line2="#1b1824",ink="#f0eef7",ink2="#bab4cc",ink3="#948ea8",dis="#5e596c",ctl="#78728e",acc="#b08cff",acct="#b996ff",onacc="#12081f",sel="#231a3a",hover="#1c1828",track="#272336",tile="#1c1828",press="#2a2538",ring="#b08cff55",err="#ff7b7b",dng="#d92d20",ondng="#ffffff",errSoft="#3a1b1b",warnSoft="#3a2d14",okt="#4ade80",chip="#231a3a",chipt="#d0b8ff",out="#1fe0f0",out2="#7af3ff",onout="#00171c",glow="#1fe0f055")),
 "seal":dict(name="Navy, seal blue, crimson pink",base="Navy blue",primary="Seal blue",outlaw="Crimson pink",
  light=dict(bg="#f1f4f8",surface="#ffffff",field="#ffffff",line="#d7dfe9",line2="#eaf0f5",ink="#101a30",ink2="#43516a",ink3="#5a687f",dis="#9ba8b9",ctl="#8090a5",acc="#3a587a",acct="#3a587a",onacc="#ffffff",sel="#e0e9f3",hover="#ebf1f7",track="#d8e0ea",tile="#ebf1f7",press="#e0e7f0",ring="#3a587a40",err="#b8400b",dng="#9f1d24",ondng="#ffffff",errSoft="#fdeee5",warnSoft="#fdf3e1",okt="#15803d",chip="#e0e9f3",chipt="#2a4462",out="#c7195e",out2="#d62a6c",onout="#ffffff",glow="transparent"),
  dark=dict(bg="#091021",surface="#0f182c",field="#0b1325",line="#20304b",line2="#16223a",ink="#e7eef7",ink2="#aebbd1",ink3="#8a98b1",dis="#55627b",ctl="#6b7a95",acc="#93b6da",acct="#93b6da",onacc="#091021",sel="#1a2a46",hover="#15213a",track="#213150",tile="#15213a",press="#1c2a45",ring="#93b6da55",err="#fb923c",dng="#c81e1e",ondng="#ffffff",errSoft="#3a2414",warnSoft="#3a2d14",okt="#4ade80",chip="#1a2a46",chipt="#b3cde8",out="#ff4f93",out2="#ff7aae",onout="#22030f",glow="#ff4f9355")),
}
PAIRS=[("Text on window","ink","bg",4.5),("Secondary text on window","ink3","bg",4.5),("Toolbar labels on panel","ink2","surface",4.5),("Primary text on panel","acct","surface",4.5),("Primary text on selected row","acct","sel",4.5),("Text on selected row","ink","sel",4.5),("Text on a pressed face","ink","press",4.5),("Text on a checked box","onacc","acc",4.5),("Outlaw button text","onout","out",4.5),("Outlaw button text, lighter end","onout","out2",4.5),("Outlaw progress on track","out","track",3),("Web chip text","chipt","chip",4.5),("Error text on window","err","bg",4.5),("Button text on danger","ondng","dng",4.5),("Checkbox and radio border","ctl","surface",3),("Primary control (focus, checked)","acc","bg",3),("Greyed text (exempt; kept readable)","dis","surface",2.3)]
def lum(h):
    h=h.lstrip('#')[:6];c=[int(h[i:i+2],16)/255 for i in (0,2,4)];c=[x/12.92 if x<=0.04045 else ((x+0.055)/1.055)**2.4 for x in c];return .2126*c[0]+.7152*c[1]+.0722*c[2]
def cr(a,b):a,b=lum(a),lum(b);return round((max(a,b)+.05)/(min(a,b)+.05),2)
def build():
    blk=lambda d:";".join(f"--{k}:{v}" for k,v in d.items())
    css=[]; rows=[]; bad=[]
    for p,P in PALS.items():
        L,D=blk(P["light"]),blk(P["dark"])
        css.append(f':root[data-pal="{p}"] .g{{{L}}}\n:root[data-pal="{p}"] .g[data-mode="dark"]{{{D};color-scheme:dark}}\n@media (prefers-color-scheme:dark){{:root[data-pal="{p}"]:not([data-theme="light"]) .g:not([data-mode="light"]){{{D};color-scheme:dark}}}}\n:root[data-pal="{p}"][data-theme="dark"] .g:not([data-mode="light"]){{{D};color-scheme:dark}}')
        css.append(f':root[data-pal="{p}"]{{--pacc:{P["light"]["acct"]};--pout:{P["light"]["out"]}}}\n@media (prefers-color-scheme:dark){{:root[data-pal="{p}"]:not([data-theme="light"]){{--pacc:{P["dark"]["acct"]};--pout:{P["dark"]["out"]}}}}}\n:root[data-pal="{p}"][data-theme="dark"]{{--pacc:{P["dark"]["acct"]};--pout:{P["dark"]["out"]}}}')
        for th in ("light","dark"):
            tk=P[th]
            for name,f,b,need in PAIRS:
                r=cr(tk[f],tk[b])
                if r<need: bad.append((p,th,name,r))
                rows.append([p,name,th.capitalize(),tk[f],tk[b],need,r])
    assert not bad,bad
    css.insert(0,".g{"+blk(PALS["graphite"]["light"])+"}")
    return "\n".join(css),rows
if __name__=="__main__":
    try: build(); print("all pass")
    except AssertionError as e: print(e)
