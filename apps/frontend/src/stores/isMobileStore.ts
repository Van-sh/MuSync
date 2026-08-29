import { create } from "zustand";

import { type Store } from "./types";

const MOBILE_BREAKPOINT = 768;

type isMobileState = {
   isMobile: boolean;
};
type isMobileActions = {
   setIsMobile: (value: boolean) => void;
};

type IsMobileStore = Store<isMobileState, isMobileActions>;

export const useIsMobileStore = create<IsMobileStore>()((set) => ({
   isMobile: false,
   actions: {
      setIsMobile(value: boolean) {
         set({ isMobile: value });
      },
   },
}));

export const useIsMobile = () => useIsMobileStore((state) => state.isMobile);

// One-time listener setup
if (typeof window !== "undefined") {
   const mql = window.matchMedia(`(max-width: ${MOBILE_BREAKPOINT - 1}px)`);
   const update = () =>
      useIsMobileStore.getState().actions.setIsMobile(window.innerWidth < MOBILE_BREAKPOINT);

   mql.addEventListener("change", update);
   update(); // initial value
}
