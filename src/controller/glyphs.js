// Button labels per controller family. Actions are positional ("a" = bottom
// face button, "b" = right, "x" = left, "y" = top), which is why Switch shows
// "B" for "a".
export const GLYPHS = {
  xbox: {
    a: "A", b: "B", x: "X", y: "Y",
    lb: "LB", rb: "RB", lt: "LT", rt: "RT",
    start: "☰", back: "⧉", guide: "⌂", rs: "RS",
  },
  playstation: {
    a: "✕", b: "○", x: "□", y: "△",
    lb: "L1", rb: "R1", lt: "L2", rt: "R2",
    start: "☰", back: "⧉", guide: "PS", rs: "R",
  },
  switch: {
    a: "B", b: "A", x: "Y", y: "X",
    lb: "L", rb: "R", lt: "ZL", rt: "ZR",
    start: "+", back: "−", guide: "⌂", rs: "R",
  },
};

export function glyphsFor(type) {
  return GLYPHS[type] || GLYPHS.xbox;
}
