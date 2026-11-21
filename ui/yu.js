/* 屿 · 交互 v0.1
   宿主里由 Rust 推 IslandState；纯浏览器里由底部控制条演示。 */
(() => {
  const body = document.body;
  const island = document.getElementById("island");
  const capsule = document.getElementById("capsule");
  const panel = document.getElementById("panel");
  const center = document.getElementById("center");
  const todoCard = document.getElementById("todoCard");
  const notices = document.getElementById("notices");
  const dock = document.getElementById("dock");

  const OCC = {
    student: ["学生", "明·课时·一矇"],
    developer: ["开发者", "渊·番茄·全图"],
    teacher: ["教师", "明·课时·一卡"],
    creator: ["创作者", "渊·自由·一卡"],
    office: ["办公", "明·番茄·全图"],
    freelance: ["自由职业", "明·短章·数列"],
  };

  let todos = [
    ["[课内] 完成山水六五作业", "课内"],
    ["[课内] 写一课胡老师数学习题", "课内"],
    ["[课内] 背《观沧海》", "课内"],
    ["[自己] 灵动岛完成", "自己"],
  ];

  const clockText = () => {
    const d = new Date();
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    const day = ["日", "一", "二", "三", "四", "五", "六"][d.getDay()];
    return [`${hh}:${mm}`, `${d.getMonth() + 1}月${d.getDate()}日 · 周${day}`];
  };

  function tick() {
    const [t, d] = clockText();
    document.getElementById("clock").textContent = t;
    document.getElementById("day").textContent = d;
    document.getElementById("panelClock").textContent = t;
    document.getElementById("panelDay").textContent = d;
    document.getElementById("centerClock").textContent = t;
    document.getElementById("centerDay").textContent = d;
  }
  tick();
  setInterval(tick, 20000);

  function ripple(x, y) {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const r = document.createElement("span");
    r.className = "ripple";
    const box = capsule.getBoundingClientRect();
    const size = 40;
    r.style.width = r.style.height = size + "px";
    r.style.left = x - box.left - size / 2 + "px";
    r.style.top = y - box.top - size / 2 + "px";
    capsule.appendChild(r);
    setTimeout(() => r.remove(), 500);
  }

  function setMode(mode, silent) {
    body.dataset.mode = mode;
    body.classList.toggle("has-notice", mode === "notice");
    if (!silent && !matchMedia("(prefers-reduced-motion: reduce)").matches) {
      body.classList.add("morph");
      setTimeout(() => body.classList.remove("morph"), 560);
    }
    dock.querySelectorAll("[data-mode]").forEach((b) => b.classList.toggle("on", b.dataset.mode === mode));
  }

  function setOccupation(id) {
    const row = OCC[id] || OCC.student;
    body.dataset.occupation = id;
    document.getElementById("occTag").textContent = row[0];
    document.getElementById("archTag").textContent = row[1];
    document.getElementById("arch").textContent = row[1];
    dock.querySelectorAll("[data-occ]").forEach((b) => b.classList.toggle("on", b.dataset.occ === id));
    setMode("occupation");
  }

  function openPanel(open) {
    panel.hidden = !open;
    island.classList.toggle("expanded", open);
    if (open) {
      const [t] = clockText();
      document.getElementById("nextThing").textContent = "数学 · 09:40";
      document.getElementById("panelClock").textContent = t;
    }
  }

  function renderTodos() {
    const ul = document.getElementById("todoList");
    ul.innerHTML = todos
      .map(
        ([title, project], i) => `<li><input type="checkbox" data-i="${i}" /><span>${title}</span><span data-project>${project}</span></li>`
      )
      .join("");
    document.getElementById("todoCount").textContent = `${todos.length} 条`;
    const badge = document.getElementById("todoBadge");
    badge.hidden = todos.length === 0;
    badge.textContent = todos.length;
  }

  function pushNotice(title, sub) {
    const el = document.createElement("div");
    el.className = "notice";
    el.innerHTML = `<span class="notice__icon"></span><div class="notice__main"><b>${title}</b><small>${sub}</small></div>`;
    notices.prepend(el);
    const nb = document.getElementById("noticeBadge");
    nb.hidden = false;
    nb.textContent = notices.children.length;
    setTimeout(() => {
      el.style.transition = "opacity .3s, transform .3s";
      el.style.opacity = "0";
      el.style.transform = "translateY(-8px)";
      setTimeout(() => {
        el.remove();
        nb.textContent = notices.children.length;
        nb.hidden = notices.children.length === 0;
      }, 320);
    }, 5200);
  }

  capsule.addEventListener("click", (e) => {
    ripple(e.clientX, e.clientY);
    openPanel(panel.hidden);
    center.hidden = true;
  });
  capsule.addEventListener("mouseenter", () => {
    if (body.dataset.mode === "idle") openPanel(true);
  });
  island.addEventListener("mouseleave", () => {
    setTimeout(() => {
      if (!island.matches(":hover")) openPanel(false);
    }, 900);
  });

  document.getElementById("todoBadge").addEventListener("click", (e) => {
    e.stopPropagation();
    todoCard.hidden = !todoCard.hidden;
    renderTodos();
  });
  document.getElementById("todoClose").addEventListener("click", () => (todoCard.hidden = true));
  document.getElementById("todoForm")?.addEventListener("submit", (e) => e.preventDefault());
  document.getElementById("todoInput").addEventListener("keydown", (e) => {
    if (e.key !== "Enter" || !e.target.value.trim()) return;
    todos.push([e.target.value.trim(), "自己"]);
    e.target.value = "";
    renderTodos();
  });
  document.getElementById("todoList").addEventListener("change", (e) => {
    if (e.target.type !== "checkbox") return;
    const i = Number(e.target.dataset.i);
    todos.splice(i, 1);
    renderTodos();
  });

  document.querySelectorAll(".panel__foot button").forEach((b) => {
    b.addEventListener("click", () => {
      if (b.dataset.act === "todo") {
        todoCard.hidden = !todoCard.hidden;
        renderTodos();
      }
      if (b.dataset.act === "center") {
        center.hidden = false;
        openPanel(false);
        const grid = document.getElementById("centerGrid");
        grid.innerHTML = ["今日完成 3/7", "专注 86 分钟", "阅读 2 次", "作业 4 项"]
          .map((x) => `<article>${x}</article>`)
          .join("");
      }
      if (b.dataset.act === "collapse") openPanel(false);
    });
  });
  document.getElementById("centerClose").addEventListener("click", () => (center.hidden = true));

  dock.addEventListener("click", (e) => {
    const m = e.target.closest("[data-mode]");
    if (m) return setMode(m.dataset.mode);
    const o = e.target.closest("[data-occ]");
    if (o) setOccupation(o.dataset.occ);
  });

  addEventListener("keydown", (e) => {
    const modes = ["idle", "media", "notice", "focus", "alert", "occupation"];
    const n = Number(e.key);
    if (n >= 1 && n <= 6) setMode(modes[n - 1]);
    if (e.key === "Escape") {
      openPanel(false);
      center.hidden = true;
      todoCard.hidden = true;
    }
  });

  /* 演示数据：指标与通知 */
  setInterval(() => {
    const cpu = 8 + Math.random() * 30;
    document.getElementById("cpuText").textContent = cpu.toFixed(0) + "%";
    document.getElementById("cpuBar").style.setProperty("--v", (cpu / 100).toFixed(2));
    document.getElementById("memText").textContent = (34 + Math.random() * 12).toFixed(0) + "%";
    document.getElementById("memBar").style.setProperty("--v", (0.42 + Math.random() * 0.08).toFixed(2));
  }, 2000);
  setTimeout(() => pushNotice("明日数学小测", "今晚 20:00 前完成复盘 · 点击转待办"), 9000);

  /* 深链：便于截图与验收 */
  const hash = location.hash.replace('#', '');
  if (hash === 'panel') openPanel(true);
  if (hash === 'center') document.querySelector('[data-act="center"]').click();
  if (hash === 'todo') { todoCard.hidden = false; renderTodos(); }
  if (hash === 'notice') { setMode('notice'); pushNotice('明日数学小测', '今晚 20:00 前完成复盘'); }

  renderTodos();
  setOccupation("student");
  setMode("idle", true);
  window.yu = { setMode, setOccupation };
})();
