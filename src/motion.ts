import { useLayoutEffect, useRef } from "react";
import { animate } from "animejs";
import { gsap } from "gsap";

export function useMotionReady() {
  return typeof window !== "undefined" && !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function useStatusMotion(active: boolean) {
  const ref = useRef<HTMLDivElement>(null);
  const motion = useMotionReady();

  useLayoutEffect(() => {
    const element = ref.current;
    if (!element || !motion) return;
    const gsapContext = gsap.context(() => {
      gsap.fromTo(element, { opacity: 0.7 }, { opacity: 1, duration: 0.18, ease: "power1.out" });
    }, element);
    const animation = animate(element, {
      translateY: ["-1px", "0px"],
      duration: 180,
      ease: "outQuad",
      autoplay: active,
    });
    return () => {
      gsapContext.revert();
      animation.pause();
    };
  }, [active, motion]);

  return ref;
}
