"use client";

import { AlertTriangle, Fingerprint, Lock, RefreshCw, Shield } from "lucide-react";
import dynamic from "next/dynamic";
import { useState } from "react";
import VulnerabilityCard from "../components/VulnerabilityCard";

// Dynamically import WalletButton to avoid SSR issues
const WalletMultiButton = dynamic(
  () => import("@solana/wallet-adapter-react-ui").then((mod) => mod.WalletMultiButton),
  { ssr: false }
);

export default function Home() {
  const [activeTab, setActiveTab] = useState(0);

  const tabs = [
    { name: "Signer Check", icon: <Lock className="w-4 h-4" /> },
    { name: "Arbitrary CPI", icon: <AlertTriangle className="w-4 h-4" /> },
    { name: "Type Cosplay", icon: <Fingerprint className="w-4 h-4" /> },
    { name: "PDA Validation", icon: <Shield className="w-4 h-4" /> },
    { name: "Re-initialization", icon: <RefreshCw className="w-4 h-4" /> },
  ];

  // Placeholder actions for demo purposes (since we lack IDL build artifacts currently)
  const mockAction = async (log: (msg: string) => void, actionName: string) => {
    log(`Initiating ${actionName}...`);
    log("Checking wallet connection...");
    await new Promise((r) => setTimeout(r, 1000));
    log("Constructing transaction...");
    await new Promise((r) => setTimeout(r, 800));
    log("Simulating signature request...");
    await new Promise((r) => setTimeout(r, 1200));
    log("✅ Transaction confirmed (Mock)");
    log("Note: Run 'anchor build' to generate IDLs for real interaction.");
  };

  return (
    <div className="min-h-screen bg-black text-gray-100 font-sans selection:bg-purple-500/30">
      {/* Navbar */}
      <nav className="border-b border-gray-800 bg-gray-900/50 backdrop-blur-md sticky top-0 z-10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="w-6 h-6 text-purple-500" />
            <span className="font-bold text-xl tracking-tight">Solana Security Templates</span>
          </div>
          <div className="bg-purple-600/10 rounded-lg p-1">
            <WalletMultiButton style={{ backgroundColor: "transparent", color: "white", height: "36px", fontSize: "14px" }} />
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="text-center mb-12">
          <h1 className="text-4xl font-extrabold mb-4 bg-linear-to-r from-purple-400 to-indigo-400 bg-clip-text text-transparent">
            Learn Solana Security
          </h1>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Interactive demonstrations of common vulnerabilities in Solana Anchor programs.
            Toggle between vulnerable and secure modes to see the difference.
          </p>
        </div>

        {/* Tabs */}
        <div className="flex flex-wrap justify-center gap-2 mb-12">
          {tabs.map((tab, idx) => (
            <button
              key={idx}
              onClick={() => setActiveTab(idx)}
              className={`flex items-center gap-2 px-6 py-3 rounded-full text-sm font-medium transition-all duration-200 ${activeTab === idx
                ? "bg-purple-600 text-white shadow-lg shadow-purple-500/25"
                : "bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white"
                }`}
            >
              {tab.icon}
              {tab.name}
            </button>
          ))}
        </div>

        {/* Content Area */}
        <div className="transition-all duration-500 ease-in-out">
          {activeTab === 0 && (
            <VulnerabilityCard
              title="Missing Signer Check"
              description="A function updates a critical authority but fails to check if the new authority signed the transaction."
              vulnerableDescription="The 'update_admin_insecure' instruction only checks if the 'admin' account key matches the config, but ignores whether it's a signer."
              secureDescription="The 'update_admin_secure' instruction uses Anchor's Signer type, enforcing that the transaction was signed by the private key."
              vulnerableAction={(log) => mockAction(log, "Exploit: Update Admin without Signature")}
              secureAction={(log) => mockAction(log, "Secure Update Admin")}
            />
          )}

          {activeTab === 1 && (
            <VulnerabilityCard
              title="Arbitrary CPI"
              description="Invoking another program without verifying its Program ID allows attackers to substitute malicious programs."
              vulnerableDescription="The 'cpi_insecure' instruction accepts any account as 'token_program', allowing calls to fake token programs."
              secureDescription="The 'cpi_secure' instruction uses Anchor's Program<'info, Token> wrapper to strictly validate the program ID."
              vulnerableAction={(log) => mockAction(log, "Exploit: Call Fake Token Program")}
              secureAction={(log) => mockAction(log, "Secure CPI Call")}
            />
          )}

          {activeTab === 2 && (
            <VulnerabilityCard
              title="Type Cosplay"
              description="Treating one account type as another because they share a similar data layout."
              vulnerableDescription="The 'cosplay_insecure' instruction manually deserializes account data without checking the Anchor discriminator."
              secureDescription="The 'cosplay_secure' instruction uses Account<'info, User>, which automatically verifies the 8-byte discriminator."
              vulnerableAction={(log) => mockAction(log, "Exploit: Pass Admin as User")}
              secureAction={(log) => mockAction(log, "Secure Type Check")}
            />
          )}

          {activeTab === 3 && (
            <VulnerabilityCard
              title="PDA Validation"
              description="Failing to validate that a PDA account was derived from the expected seeds."
              vulnerableDescription="The 'deposit_insecure' instruction accepts any 'Pool' account, allowing attackers to supply their own fake pool."
              secureDescription="The 'deposit_secure' instruction uses the 'seeds' constraint to enforce the exact PDA address derivation."
              vulnerableAction={(log) => mockAction(log, "Exploit: Deposit to Fake Pool")}
              secureAction={(log) => mockAction(log, "Secure Deposit")}
            />
          )}

          {activeTab === 4 && (
            <VulnerabilityCard
              title="Re-initialization"
              description="Initializing an account without checking if it has already been initialized."
              vulnerableDescription="The 'initialize_insecure' instruction blindly writes data to the account, allowing overwrites."
              secureDescription="The 'initialize_secure' instruction uses the 'init' constraint to ensure the account is fresh."
              vulnerableAction={(log) => mockAction(log, "Exploit: Overwrite Existing Account")}
              secureAction={(log) => mockAction(log, "Secure Initialization")}
            />
          )}
        </div>
      </main>
    </div>
  );
}
