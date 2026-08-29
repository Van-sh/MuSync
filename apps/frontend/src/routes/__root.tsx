import { TanStackDevtools } from "@tanstack/react-devtools";
import { type QueryClient } from "@tanstack/react-query";
import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { HeadContent, Scripts, createRootRouteWithContext } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { ThemeProvider } from "better-themes";

import appCss from "../styles.css?url";

interface MyRouterContext {
   queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
   head: () => ({
      meta: [
         {
            charSet: "utf-8",
         },
         {
            name: "viewport",
            content: "width=device-width, initial-scale=1",
         },
         {
            title: "TanStack Start Starter",
         },
      ],
      links: [
         {
            rel: "stylesheet",
            href: appCss,
         },
      ],
   }),
   shellComponent: RootDocument,
});

function RootDocument({ children }: { children: React.ReactNode }) {
   return (
      <html lang="en">
         <head>
            <HeadContent />
         </head>
         <body>
            <ThemeProvider attribute="class" disableTransitionOnChange>
               {children}
               <TanStackDevtools
                  config={{
                     position: "bottom-right",
                  }}
                  plugins={[
                     {
                        name: "Tanstack Router",
                        render: <TanStackRouterDevtoolsPanel />,
                     },
                     {
                        name: "Tanstack Query",
                        render: <ReactQueryDevtoolsPanel />,
                     },
                  ]}
               />
            </ThemeProvider>
            <Scripts />
         </body>
      </html>
   );
}
