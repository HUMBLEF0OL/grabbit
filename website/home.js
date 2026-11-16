// The Grabbit home page's motion (home.css styles it; main.js keeps the theme button and the site's
// shared behaviour). Every part shows its finished state without this script or with reduced motion.
(function () {
  "use strict";
  var reduce = window.matchMedia && matchMedia("(prefers-reduced-motion: reduce)").matches;
  var IO = "IntersectionObserver" in window;

  // The benchmark chart's bars grow once when the chart comes into view (they rest full by default).
  var bench = document.querySelector("[data-bench]");
  if (bench) {
    if (reduce || !IO) bench.classList.add("in");
    else new IntersectionObserver(function (es, io) {
      es.forEach(function (e) { if (e.isIntersecting) { bench.classList.add("in"); io.disconnect(); } });
    }, { threshold: 0.3 }).observe(bench);
  }

  // The statement lights word by word as it scrolls past (CSS scroll-driven; this only numbers the words).
  var say = document.getElementById("say");
  if (say) {
    var i = 0;
    say.querySelectorAll(".c").forEach(function (c) {
      c.outerHTML = c.textContent.split(" ").map(function (w) { return '<span class="c">' + w + "</span>"; }).join(" ");
    });
    Array.prototype.slice.call(say.childNodes).forEach(function (n) {
      if (n.nodeType !== 3) return;
      var f = document.createDocumentFragment();
      n.textContent.split(/(\s+)/).forEach(function (w) {
        if (/\S/.test(w)) { var s = document.createElement("span"); s.textContent = w; f.appendChild(s); }
        else f.appendChild(document.createTextNode(w));
      });
      n.parentNode.replaceChild(f, n);
    });
    say.querySelectorAll("span").forEach(function (s) { s.style.setProperty("--i", i++); });
  }

  // The connections cell: six parts fill at their own pace in real time, the last at 1.01 s (the
  // benchmark's median), hold on "Complete", then replay while the cell is in view.
  var cell = document.getElementById("race");
  if (cell && !reduce && IO) {
    var bars = Array.prototype.slice.call(cell.querySelectorAll(".conns i"));
    var t = document.getElementById("rt"), st = document.getElementById("rs");
    var PRE = 500, END = 1010, HOLD = 1800, t0 = 0, raf = 0, seen = false;
    var ease = function (x) { return 1 - Math.pow(1 - x, 1.35); };
    var paint = function (k, label, ok) {
      t.textContent = k; st.textContent = label; st.classList.toggle("ok", ok);
    };
    var frame = function (now) {
      var e = now - t0 - PRE;
      if (e < 0) {
        bars.forEach(function (b) { b.style.setProperty("--k", 0); b.classList.remove("done"); });
        paint("0.00 s", "Starting", false);
      } else if (e <= END) {
        bars.forEach(function (b) {
          var k = Math.min(1, e / +b.getAttribute("data-end"));
          b.style.setProperty("--k", ease(k)); b.classList.toggle("done", k >= 1);
        });
        paint((e / 1000).toFixed(2) + " s", "Downloading", false);
      } else if (e <= END + HOLD) {
        bars.forEach(function (b) { b.style.setProperty("--k", 1); b.classList.add("done"); });
        paint("1.01 s", "Complete", true);
      } else t0 = now;
      raf = seen ? requestAnimationFrame(frame) : 0;
    };
    new IntersectionObserver(function (es) {
      seen = es[0].isIntersecting;
      if (seen && !raf) { t0 = performance.now(); raf = requestAnimationFrame(frame); }
    }, { threshold: 0.4 }).observe(cell);
  } else if (cell) {
    cell.querySelectorAll(".conns i").forEach(function (b) { b.classList.add("done"); });
  }

  // The pinned walk-through: each step marker, crossing the middle of the screen, selects its rail
  // item and window. Under 1000px the steps stack and the stage is hidden (home.css).
  var steps = document.querySelectorAll(".step"), shots = document.querySelectorAll(".shot");
  if (steps.length && IO) {
    var io = new IntersectionObserver(function (es) {
      es.forEach(function (e) {
        if (!e.isIntersecting) return;
        var k = +e.target.getAttribute("data-s");
        [steps, shots].forEach(function (list) {
          list.forEach(function (el, j) { el.classList.toggle("on", j === k); });
        });
      });
    }, { rootMargin: "-50% 0px -50% 0px" });
    document.querySelectorAll(".steps div").forEach(function (s) { io.observe(s); });
  }
})();
