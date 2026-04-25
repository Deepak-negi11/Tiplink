import { NextRequest, NextResponse } from "next/server";
import Stripe from "stripe";

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: "2025-03-31.basil",
});

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const { amount, currency, walletAddress, userId } = body;

    if (!amount || !walletAddress) {
      return NextResponse.json({ error: "Missing amount or wallet address" }, { status: 400 });
    }

    const session = await stripe.checkout.sessions.create({
      payment_method_types: ["card"],
      line_items: [
        {
          price_data: {
            currency: currency || "usd",
            product_data: {
              name: "SOL Deposit",
              description: `Deposit to wallet ${walletAddress.slice(0, 8)}...${walletAddress.slice(-4)}`,
            },
            unit_amount: Math.round((amount || 50) * 100),
          },
          quantity: 1,
        },
      ],
      mode: "payment",
      success_url: `${req.nextUrl.origin}/dashboard?deposit=success&session_id={CHECKOUT_SESSION_ID}`,
      cancel_url: `${req.nextUrl.origin}/dashboard/deposit?cancelled=true`,
      metadata: {
        walletAddress,
        userId: userId || "",
        tokenType: "SOL",
      },
    });

    return NextResponse.json({ url: session.url, sessionId: session.id });
  } catch (err: any) {
    console.error("Stripe checkout error:", err.message);
    return NextResponse.json({ error: err.message }, { status: 500 });
  }
}
