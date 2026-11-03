(() => {
  const age = document.getElementById("age");
  const subject = document.getElementById("subject");
  const count = document.getElementById("count");
  const empty = document.getElementById("empty");
  const rows = document.querySelectorAll("#find [data-subjects]");
  function update() {
    const a = Number(age.value);
    const s = subject.value;
    let shown = 0;
    rows.forEach((row) => {
      const d = row.dataset;
      const ok =
        (age.value === "" || (a >= Number(d.ageMin) && a <= Number(d.ageMax))) &&
        (s === "anything" || d.subjects.split("|").includes(s));
      row.hidden = !ok;
      if (ok) shown++;
    });
    count.textContent = shown;
    empty.hidden = shown > 0;
  }
  age.addEventListener("change", update);
  subject.addEventListener("change", update);
  update();
})();
