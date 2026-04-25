import { NextResponse } from "next/server";
import Stripe from "stripe";

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY as string, {
  apiVersion: "2025-02-24.acacia" as any, // latest API version
});

export async function POST(req: Request) {
  try {
    const body = await req.json();
    const { walletAddress } = body;

    if (!walletAddress) {
      return NextResponse.json({ error: "walletAddress is required" }, { status: 400 });
    }

    if (!process.env.STRIPE_SECRET_KEY) {
      return NextResponse.json({ error: "STRIPE_SECRET_KEY is not set" }, { status: 500 });
    }

    const response = await fetch("https://api.stripe.com/v1/crypto/onramp_sessions", {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${process.env.STRIPE_SECRET_KEY}`,
        "Content-Type": "application/x-www-form-urlencoded",
        "Stripe-Version": "2024-06-20",
      },
      body: new URLSearchParams({
        "destination_currencies": "sol,usdc",
        "destination_networks": "solana",
        "wallet_addresses[solana]": walletAddress,
      }),
    });

    const session = await response.json();
    
    if (!response.ok) {
      throw new Error(session.error?.message || "Failed to create Stripe session");
    }

    return NextResponse.json({ clientSecret: session.client_secret });
  } catch (error: any) {
    console.error("Stripe onramp error:", error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
