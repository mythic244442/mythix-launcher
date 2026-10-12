// controller/overlays.js — open/close state for the on-screen keyboard and the
// option picker, plus helpers to write into React-controlled fields.

import { useSyncExternalStore } from "react";

let snapshot = { keyboard: null, picker: null };
const subscribers = new Set();
let openSeq = 0;

function set(patch) {
  snapshot = { ...snapshot, ...patch };
  subscribers.forEach((fn) => fn());
}

function subscribe(fn) {
  subscribers.add(fn);
  return () => subscribers.delete(fn);
}

export function useOverlays() {
  return useSyncExternalStore(subscribe, () => snapshot, () => snapshot);
}

export function getOverlays() {
  return snapshot;
}

export function isTextField(el) {
  if (el instanceof HTMLTextAreaElement) return !el.readOnly && !el.disabled;
  if (!(el instanceof HTMLInputElement) || el.readOnly || el.disabled) return false;
  return ["text", "search", "url", "email", "tel", "password", "number", ""].includes(el.type);
}

export function openKeyboard(target) {
  if (!isTextField(target)) return;
  set({ keyboard: { target, id: ++openSeq } });
}

export function closeKeyboard({ refocus = true } = {}) {
  const target = snapshot.keyboard?.target;
  if (!snapshot.keyboard) return;
  set({ keyboard: null });
  if (refocus && target?.isConnected) target.focus({ preventScroll: true });
}

export function openPicker(select) {
  if (!(select instanceof HTMLSelectElement) || select.disabled || select.options.length === 0) return;
  set({ picker: { select, id: ++openSeq } });
}

export function closePicker({ refocus = true } = {}) {
  const select = snapshot.picker?.select;
  if (!snapshot.picker) return;
  set({ picker: null });
  if (refocus && select?.isConnected) select.focus({ preventScroll: true });
}

/**
 * Set a value the way a user edit would, so React's onChange fires.
 * (Assigning el.value directly is swallowed by React's value tracker.)
 */
export function setNativeValue(el, value) {
  const proto =
    el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype
    : el instanceof HTMLSelectElement ? HTMLSelectElement.prototype
    : HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
  if (setter) setter.call(el, value);
  else el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  if (el instanceof HTMLSelectElement) el.dispatchEvent(new Event("change", { bubbles: true }));
}

/** Best-effort human label for a form field. */
export function fieldLabel(el) {
  if (!el) return "";
  const aria = el.getAttribute("aria-label");
  if (aria) return aria;
  if (el.id) {
    try {
      const label = document.querySelector(`label[for="${CSS.escape(el.id)}"]`);
      if (label?.textContent.trim()) return label.textContent.trim();
    } catch (_) { /* ignore */ }
  }
  const groupLabel = el.closest(".form-group")?.querySelector(".form-label");
  if (groupLabel?.textContent.trim()) return groupLabel.textContent.trim();
  // SettingsView rows: <div><div>Label</div>[<div>help</div>]<input/></div>
  // (skip icon-only siblings like the search box's "⌕")
  const first = el.parentElement?.firstElementChild;
  const text = first && first !== el && !first.contains(el) ? first.textContent.trim() : "";
  if (/[a-z]{2}/i.test(text) && text.length <= 80) return text;
  return el.getAttribute("placeholder") || "Text";
}
