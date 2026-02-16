"use client";

import { QueryClient, QueryClientProvider as QueryProvider } from "@tanstack/react-query";

export default function QueryClientProvider({ children }: React.PropsWithChildren) {
   return <QueryProvider client={new QueryClient()}>{children}</QueryProvider>;
}
