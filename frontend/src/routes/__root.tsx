import { TanStackDevtools } from "@tanstack/react-devtools";
import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { createRootRouteWithContext, HeadContent, Link, Scripts } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { ThemeProvider } from "better-themes";

import { Toaster } from "~/components/ui/sonner";
import { TooltipProvider } from "~/components/ui/tooltip";
import { IsMobileProvider } from "~/context/IsMobileContext";
import { env } from "~/env";
import type { RouterContext } from "~/router.context";

import appCss from "~/styles.css?url";

export const Route = createRootRouteWithContext<RouterContext>()({
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
            title: env.VITE_APP_TITLE,
         },
      ],
      links: [
         {
            rel: "stylesheet",
            href: appCss,
         },
         {
            rel: "icon",
            href: "favicon.svg",
         },
      ],
   }),
   shellComponent: RootDocument,
   notFoundComponent: NotFound,
});

function RootDocument({ children }: { children: React.ReactNode }) {
   return (
      <html lang="en" suppressHydrationWarning>
         <head>
            <HeadContent />
         </head>
         <body className="h-svh">
            <ThemeProvider attribute="class" disableTransitionOnChange>
               <IsMobileProvider>
                  <TooltipProvider>{children}</TooltipProvider>
                  <Toaster position="bottom-right" closeButton richColors />
                  <TanStackDevtools
                     config={{
                        position: "bottom-left",
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
                  <Scripts />
               </IsMobileProvider>
            </ThemeProvider>
         </body>
      </html>
   );
}

function NotFound() {
   return (
      <main className="flex min-h-[60vh] items-center justify-center px-4">
         <div className="space-y-4 text-center">
            <p className="text-sm font-semibold tracking-widest text-cyan-400">404</p>
            <h1 className="text-3xl font-bold tracking-tight text-slate-50 md:text-4xl">
               Page not found
            </h1>
            <p className="mx-auto max-w-md text-sm text-slate-400 md:text-base">
               The page you’re looking for doesn’t exist or may have been moved.
            </p>
            <div className="flex justify-center">
               <Link
                  to="/"
                  className="inline-flex items-center rounded-md bg-cyan-500 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-cyan-600 focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950 focus-visible:outline-none"
               >
                  Go back home
               </Link>
            </div>
         </div>
      </main>
   );
}
