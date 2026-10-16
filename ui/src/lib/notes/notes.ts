/**
 * Account notes are stored as a small subset of HTML written by the editor.
 * ! Every note goes through `sanitizeNote` before it reaches `innerHTML` or the
 * ! backend: only the tags below survive, and colours must be plain hex values.
 * ! The backend refuses any other markup, so the two must stay in step.
 */

/// Mirrors `MAX_NOTE_BYTES` in the backend's settings model.
export const MAX_NOTE_BYTES = 64 * 1024;

export const noteBytes = (html: string): number => new TextEncoder().encode(html).length;

export const NOTE_COLORS: string[] = [
  "#f43f5e",
  "#f97316",
  "#f59e0b",
  "#84cc16",
  "#10b981",
  "#06b6d4",
  "#3b82f6",
  "#8b5cf6",
  "#ec4899",
  "#8f98a0",
];

const SIMPLE_TAGS: Record<string, string> = {
  B: "b",
  STRONG: "b",
  I: "i",
  EM: "i",
  U: "u",
  S: "s",
  STRIKE: "s",
  DEL: "s",
  DIV: "div",
  P: "div",
  UL: "ul",
  OL: "ol",
  LI: "li",
};

/// Their content is code or styling, never text the user typed.
const DROPPED_TAGS = new Set(["SCRIPT", "STYLE", "TEMPLATE", "NOSCRIPT"]);

const escapeText = (value: string): string =>
  value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

export const toHex = (value: string): string | null => {
  const text = value.trim().toLowerCase();
  if (/^#[0-9a-f]{6}$/.test(text)) return text;
  if (/^#[0-9a-f]{3}$/.test(text))
    return `#${text
      .slice(1)
      .split("")
      .map((digit) => digit + digit)
      .join("")}`;
  const rgb = text.match(/^rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})/);
  if (!rgb) return null;
  return `#${rgb
    .slice(1, 4)
    .map((part) => Math.min(255, Number(part)).toString(16).padStart(2, "0"))
    .join("")}`;
};

/// The editor writes `<font color>` or a `color` style depending on the engine.
const colorOf = (element: Element): string | null => {
  const attribute = element.getAttribute("color");
  if (attribute) return toHex(attribute);
  return element instanceof HTMLElement && element.style.color ? toHex(element.style.color) : null;
};

/// Engines that ignore `styleWithCSS` write the formatting as inline styles.
const styleTags = (element: Element): string[] => {
  if (!(element instanceof HTMLElement)) return [];
  const { fontWeight, fontStyle, textDecorationLine, textDecoration } = element.style;
  const decoration = `${textDecorationLine} ${textDecoration}`;
  const tags: string[] = [];
  if (fontWeight === "bold" || Number(fontWeight) >= 600) tags.push("b");
  if (fontStyle === "italic") tags.push("i");
  if (decoration.includes("underline")) tags.push("u");
  if (decoration.includes("line-through")) tags.push("s");
  return tags;
};

//! A colour is only ever written around text, never around a block, so the
//! output nests validly and parses back to the very same markup.
const serialize = (node: Node, color: string | null): string => {
  if (node.nodeType === Node.TEXT_NODE) {
    const text = escapeText(node.textContent ?? "");
    return color && text ? `<span style="color:${color}">${text}</span>` : text;
  }
  if (!(node instanceof Element)) return "";
  // `tagName` keeps the source case for SVG and MathML elements.
  const name = node.localName.toUpperCase();
  if (name === "BR") return "<br>";
  if (DROPPED_TAGS.has(name)) return "";
  const own = colorOf(node) ?? color;
  const inner = [...node.childNodes].map((child) => serialize(child, own)).join("");
  const simple = SIMPLE_TAGS[name];
  const tags = [...(simple ? [simple] : []), ...styleTags(node)];
  return tags.reduceRight((html, tag) => `<${tag}>${html}</${tag}>`, inner);
};

export const sanitizeNote = (html: string | null | undefined): string => {
  if (!html) return "";
  // DOMParser never runs scripts or loads resources from what it parses.
  const body = new DOMParser().parseFromString(html, "text/html").body;
  return [...body.childNodes].map((child) => serialize(child, null)).join("");
};
