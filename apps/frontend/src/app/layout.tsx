import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

import Navbar from "~/components/Navbar";
import QueryClientProvider from "~/components/QueryProvider";
import ThemeProvider from "~/components/ThemeProvider";
import { cn } from "~/lib/utils";

import "./globals.css";

const geistSans = Geist({
   variable: "--font-geist-sans",
   subsets: ["latin"],
});

const geistMono = Geist_Mono({
   variable: "--font-geist-mono",
   subsets: ["latin"],
});

export const metadata: Metadata = {
   title: "MuSync",
   description: "App for listening to music",
};

export default function RootLayout({
   children,
}: Readonly<{
   children: React.ReactNode;
}>) {
   return (
      <html lang="en" suppressHydrationWarning>
         <body
            className={cn(
               geistSans.variable,
               geistMono.variable,
               "flex h-screen flex-col antialiased",
            )}
         >
            <QueryClientProvider>
               <ThemeProvider
                  attribute="class"
                  defaultTheme="system"
                  enableSystem
                  disableTransitionOnChange
               >
                  <Navbar />

                  <main className="flex h-full w-full grow flex-col">{children}</main>
               </ThemeProvider>
            </QueryClientProvider>
         </body>
      </html>
   );
}
