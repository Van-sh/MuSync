import { QueryClient } from "@tanstack/react-query";

export interface RouterContext {
   queryClient: QueryClient;
}
let context: RouterContext | undefined;
export function getContext(): RouterContext {
   if (context) {
      return context;
   }

   const queryClient = new QueryClient({
      defaultOptions: {
         queries: {
            refetchOnWindowFocus: true,
         },
      },
   });

   context = { queryClient } satisfies RouterContext;
   return context;
}
