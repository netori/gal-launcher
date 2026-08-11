/**
 * 迷你 markdown 渲染：先转义再变换，产物只有自产标签（<strong>/<code>/<a>/<h3>/<ul>/<li>/<p>），
 * 数据源是自家 release body，双重保障无注入。渲染出的链接自动带 target="_blank" rel="noopener"。
 */

const esc = (s: string): string =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function inline(s: string): string {
  return esc(s)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(
      /\[([^\]]+)\]\(([^)\s]+)\)/g,
      '<a href="$2" target="_blank" rel="noopener">$1</a>',
    );
}

export function renderChangelog(body: string): string {
  const out: string[] = [];
  let inList = false;
  const closeList = () => {
    if (inList) {
      out.push("</ul>");
      inList = false;
    }
  };

  for (const raw of body.replace(/\r/g, "").split("\n")) {
    const t = raw.trim();
    if (!t) {
      closeList();
      continue;
    }
    const head = t.match(/^#{1,4}\s+(.*)$/);
    if (head) {
      closeList();
      out.push(`<h3>${inline(head[1])}</h3>`);
      continue;
    }
    const item = t.match(/^[-*]\s+(.*)$/);
    if (item) {
      if (!inList) {
        out.push("<ul>");
        inList = true;
      }
      out.push(`<li>${inline(item[1])}</li>`);
      continue;
    }
    closeList();
    out.push(`<p>${inline(t)}</p>`);
  }
  closeList();
  return out.join("");
}
