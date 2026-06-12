import { useEffect, useRef } from "react";

const DEADZONE = 0.5;
const REPEAT_DELAY = 400;
const REPEAT_RATE = 150;

const DIRS = ["up", "down", "left", "right"];

export function useGamepad(onInput) {
  const cbRef = useRef(onInput);
  cbRef.current = onInput;

  const prev = useRef({});
  const repeatTimers = useRef({});
  const throttle = useRef(0);

  useEffect(() => {
    let raf;
    let active = true;

    function startRepeat(action) {
      if (!DIRS.includes(action)) return;
      stopRepeat(action);
      repeatTimers.current[action] = setTimeout(() => {
        repeatTimers.current[action] = setInterval(() => {
          cbRef.current?.(action);
        }, REPEAT_RATE);
      }, REPEAT_DELAY);
    }

    function stopRepeat(action) {
      clearTimeout(repeatTimers.current[action]);
      clearInterval(repeatTimers.current[action]);
      delete repeatTimers.current[action];
    }

    function poll() {
      if (!active) return;
      raf = requestAnimationFrame(poll);

      const now_ms = performance.now();
      if (now_ms - throttle.current < 16) return;
      throttle.current = now_ms;

      const gamepads = navigator.getGamepads?.();
      if (!gamepads) return;

      let gp = null;
      for (let i = 0; i < gamepads.length; i++) {
        if (gamepads[i]) { gp = gamepads[i]; break; }
      }
      if (!gp) return;

      const lx = gp.axes[0] ?? 0;
      const ly = gp.axes[1] ?? 0;
      const btns = gp.buttons;

      const now = {
        up:    ly < -DEADZONE || !!btns[12]?.pressed,
        down:  ly > DEADZONE  || !!btns[13]?.pressed,
        left:  lx < -DEADZONE || !!btns[14]?.pressed,
        right: lx > DEADZONE  || !!btns[15]?.pressed,
        a:     !!btns[0]?.pressed,
        b:     !!btns[1]?.pressed,
        x:     !!btns[2]?.pressed,
        y:     !!btns[3]?.pressed,
        lb:    !!btns[4]?.pressed,
        rb:    !!btns[5]?.pressed,
      };

      // Guide button — index 16, may not exist on all controllers
      if (btns.length > 16 && btns[16]) {
        now.guide = !!btns[16].pressed;
      } else {
        now.guide = false;
      }

      for (const action in now) {
        const wasDown = !!prev.current[action];
        const isDown = now[action];

        if (isDown && !wasDown) {
          try { cbRef.current?.(action); } catch (_) {}
          startRepeat(action);
        } else if (!isDown && wasDown) {
          stopRepeat(action);
        }
      }

      prev.current = now;
    }

    raf = requestAnimationFrame(poll);

    return () => {
      active = false;
      cancelAnimationFrame(raf);
      for (const action in repeatTimers.current) {
        clearTimeout(repeatTimers.current[action]);
        clearInterval(repeatTimers.current[action]);
      }
      repeatTimers.current = {};
      prev.current = {};
    };
  }, []);
}
