import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({ variable: "--font-geist-sans", subsets: ["latin"] });
const geistMono = Geist_Mono({ variable: "--font-geist-mono", subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Palinurus — the Solana DePIN node that talks",
  description:
    "A $40 Raspberry Pi running ZeroClaw becomes a Solana-attesting DePIN device. Two WIT plugins — depin-attest (sensor → Solana Attestation Service) and depin-rewards (watch any public Helium hotspot, get a Telegram alert when it goes dark). The agent never holds a main wallet key.",
  keywords: [
    "Solana", "DePIN", "ZeroClaw", "Helium", "Solana Attestation Service",
    "WIT", "wasm32-wasip2", "Raspberry Pi", "Squads multisig",
  ],
  openGraph: {
    title: "Palinurus — the Solana DePIN node that talks",
    description:
      "Two ZeroClaw WIT plugins that turn a Raspberry Pi into a Solana-attesting DePIN node — and watch your Helium hotspot from your laptop. Agent proposes, multisig disposes.",
    type: "website",
    url: "https://palinurus.rectorspace.com",
    siteName: "Palinurus",
  },
  twitter: { card: "summary_large_image", title: "Palinurus — the Solana DePIN node", description: "Two ZeroClaw WIT plugins · Solana DePIN · agent proposes, multisig disposes." },
  authors: [{ name: "RECTOR-LABS" }],
  metadataBase: new URL("https://palinurus.rectorspace.com"),
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}>
      <body className="min-h-full flex flex-col">{children}</body>
    </html>
  );
}