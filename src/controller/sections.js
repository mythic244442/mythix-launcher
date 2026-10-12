// The four launcher sections, in the order LB / RB walk them.
// Maximized mode draws them as a pinwheel around the content
// (Library top → Import right → Settings bottom → Tools left), so RB moves
// clockwise and LB counter-clockwise. Reorder here if the frame changes.
export const SECTION_ORDER = ["library", "steam", "settings", "tools"];

export function neighborSection(view, step) {
  const i = SECTION_ORDER.indexOf(view);
  const n = SECTION_ORDER.length;
  return SECTION_ORDER[((i < 0 ? 0 : i) + step + n) % n];
}
