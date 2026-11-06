(() => {
  const $ = (s) => [...document.querySelectorAll(s)];
  const cards = $("#grid > li");
  const rows = $("#cover li[data-for]");
  const chips = $(".chip");
  const age = document.getElementById("age");
  const clear = document.getElementById("clear");
  const on = {};
  const changed = (axis, value) => document.dispatchEvent(
    new CustomEvent("mercury:filter_changed", { detail: { axis, value } }),
  );
  chips.forEach((b) => (on[b.dataset.axis] = new Set()));
  const has = (c, axis, set) =>
    !set.size || c.dataset[axis].split("|").some((v) => set.has(v));
  const fits = (c) =>
    !age.value || (+age.value >= +c.dataset.ageMin && +age.value <= +c.dataset.ageMax);
  // `skip` leaves one axis out so each chip counts what it would add.
  const match = (c, skip) =>
    fits(c) && Object.keys(on).every((a) => a === skip || has(c, a, on[a]));
  function update() {
    const shown = new Set(cards.filter((c) => !(c.hidden = !match(c))).map((c) => c.id));
    rows.forEach((r) => r.classList.toggle("off", !shown.has(r.dataset.for)));
    chips.forEach((b) => {
      const { axis, value } = b.dataset;
      const one = new Set([value]);
      const n = cards.filter((c) => match(c, axis) && has(c, axis, one)).length;
      b.lastChild.textContent = n;
      b.disabled = !n && !on[axis].has(value);
    });
    $("#cover .cells > *").forEach((x, i) =>
      x.classList.toggle("now", age.value !== "" && i % 16 === age.value - 3),
    );
    document.getElementById("count").textContent = shown.size;
    document.getElementById("empty").hidden = shown.size > 0;
    clear.hidden = !age.value && Object.values(on).every((s) => !s.size);
  }
  chips.forEach((b) =>
    b.addEventListener("click", () => {
      const set = on[b.dataset.axis];
      const v = b.dataset.value;
      set.has(v) ? set.delete(v) : set.add(v);
      b.setAttribute("aria-pressed", set.has(v));
      update();
      changed(b.dataset.axis, set.has(v) ? v : "");
    }),
  );
  clear.addEventListener("click", () => {
    age.value = "";
    Object.values(on).forEach((s) => s.clear());
    chips.forEach((b) => b.setAttribute("aria-pressed", false));
    update();
    changed("all", "");
  });
  age.addEventListener("input", () => {
    update();
    changed("age", age.value);
  });
  update();
})();
