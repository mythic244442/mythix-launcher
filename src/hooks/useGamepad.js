import { useEffect, useRef, useSyncExternalStore } from "react";
import { addLayer, subscribe, getSnapshot } from "../controller/bus.js";

/**
 * Register an input layer for as long as the component is mounted (and
 * `enabled`). Only the highest-priority layer receives presses, so an open
 * overlay automatically mutes everything underneath it.
 *
 *   onPress(action, { repeat })   handler; held directions auto-repeat
 *   options.priority              base nav 0 · quick settings 50 · picker 90 · keyboard 100
 *   options.repeat                extra actions that auto-repeat while held
 *   options.onRelease(action)     optional release handler
 */
export function useGamepadLayer(onPress, { priority = 50, enabled = true, repeat, onRelease } = {}) {
  const pressRef = useRef(onPress);
  const releaseRef = useRef(onRelease);
  pressRef.current = onPress;
  releaseRef.current = onRelease;
  const repeatKey = repeat ? repeat.join(",") : "";

  useEffect(() => {
    if (!enabled) return undefined;
    const layer = addLayer({
      priority,
      repeat: repeatKey ? repeatKey.split(",") : [],
      onPress: (action, info) => pressRef.current?.(action, info),
      onRelease: (action) => releaseRef.current?.(action),
    });
    return layer.remove;
  }, [priority, enabled, repeatKey]);
}

/** { type: "xbox"|"playstation"|"switch", connected: number, active: boolean } */
export function useControllerState() {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

/**
 * Back-compat wrapper (QuickSettings uses this): an exclusive layer above the
 * base navigation. Returns the controller glyph family.
 */
export function useGamepad(onInput, options) {
  useGamepadLayer(onInput, { priority: 50, ...options });
  return useControllerState().type;
}
