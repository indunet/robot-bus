const install = document.querySelector("[data-install]");
const tablist = install.querySelector("[role=tablist]");
const tabs = Array.from(tablist.querySelectorAll("[role=tab]"));
const copyStatus = install.querySelector("[data-copy-status]");

function selectTab(selected) {
  for (const tab of tabs) {
    const active = tab === selected;
    tab.setAttribute("aria-selected", String(active));
    tab.tabIndex = active ? 0 : -1;
    document.getElementById(tab.getAttribute("aria-controls")).hidden = !active;
  }
}

for (const tab of tabs) {
  tab.addEventListener("click", () => selectTab(tab));
}

tablist.addEventListener("keydown", (event) => {
  const index = tabs.indexOf(event.target);
  let next;

  switch (event.key) {
    case "ArrowLeft":
      next = (index - 1 + tabs.length) % tabs.length;
      break;
    case "ArrowRight":
      next = (index + 1) % tabs.length;
      break;
    case "Home":
      next = 0;
      break;
    case "End":
      next = tabs.length - 1;
      break;
    default:
      return;
  }

  event.preventDefault();
  selectTab(tabs[next]);
  tabs[next].focus();
});

if (navigator.clipboard?.writeText) {
  for (const button of install.querySelectorAll(".copy")) {
    const code = button.closest(".sdk-panel").querySelector("code");
    let reset;

    button.hidden = false;
    button.addEventListener("click", async () => {
      clearTimeout(reset);
      copyStatus.textContent = "";
      try {
        await navigator.clipboard.writeText(code.textContent);
        button.textContent = install.dataset.copied;
      } catch {
        button.textContent = install.dataset.copyError;
      }
      copyStatus.textContent = button.textContent;
      reset = setTimeout(() => {
        button.textContent = install.dataset.copy;
      }, 1800);
    });
  }
}

selectTab(tabs[0]);
tablist.hidden = false;
