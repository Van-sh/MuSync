import { createContext, useContext, useEffect, useState, type PropsWithChildren } from "react";

const MOBILE_BREAKPOINT = 768;

export const IsMobileContext = createContext<boolean | null>(null);

export function IsMobileProvider({ children }: PropsWithChildren) {
   const [isMobile, setIsMobile] = useState<boolean | undefined>(undefined);

   useEffect(() => {
      const mql = window.matchMedia(`(max-width: ${MOBILE_BREAKPOINT - 1}px)`);
      const onChange = () => {
         setIsMobile(window.innerWidth < MOBILE_BREAKPOINT);
      };
      mql.addEventListener("change", onChange);
      setIsMobile(window.innerWidth < MOBILE_BREAKPOINT);
      return () => mql.removeEventListener("change", onChange);
   }, []);

   return <IsMobileContext.Provider value={isMobile!}>{children}</IsMobileContext.Provider>;
}

export function useIsMobile() {
   const isMobile = useContext(IsMobileContext);

   if (isMobile === null) console.error("useIsMobile must only be used inside IsMobileProvider");

   return isMobile;
}
