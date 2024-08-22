import type { Metadata } from "next";
import {Space_Grotesk} from "next/font/google"

import "./globals.css";

const spaceGrotesk = Space_Grotesk({
  subsets: ['latin'],
  style: 'normal'
})


export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const rootClass = spaceGrotesk.className + " bg-black text-white";
  return (
    <html lang="en" className={rootClass} >
      <body>{children}</body>
    </html>
  );
}
