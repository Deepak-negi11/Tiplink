"use client";
import { useState, useEffect } from "react";
import { useAuthStore } from "@/store/useStore";
import { QRCodeSVG } from "qrcode.react";
import { fetchApi } from "@/lib/api";

interface Quote {
    quote_currency_amount: number;
    fee_amount: number;
    total_amount: number;
}

interface Limits {
    base_min_amount: number;
    base_max_amount: number;
}

export default function DepositPage() {
    const { user, token } = useAuthStore();
    const [tab, setTab] = useState<"buy" | "transfer">("buy");
    const [amount, setAmount] = useState("50");
    const [currency, setCurrency] = useState("sol");
    const [quote, setQuote] = useState<Quote | null>(null);
    const [limits, setLimits] = useState<Limits | null>(null);
    const [loading, setLoading] = useState(false);
    const [copied, setCopied] = useState(false);
    const [showWidget, setShowWidget] = useState(false);

    const walletAddress = user?.public_key ?? "";

    // fetch limits on load
    useEffect(() => {
        fetchApi<Limits>(`/moonpay/limits?currency_code=${currency}`, { token })
            .then(data => setLimits(data))
            .catch(console.error);
    }, [currency]);

    // fetch quote when amount changes
    useEffect(() => {
        if (!amount || parseFloat(amount) < (limits?.base_min_amount ?? 0)) {
            setQuote(null);
            return;
        }
        const timer = setTimeout(async () => {
            try {
                const data = await fetchApi<Quote>(
                    `/moonpay/quote?currency_code=${currency}&fiat_currency=usd&fiat_amount=${amount}`,
                    { token }
                );
                setQuote(data);
            } catch { setQuote(null); }
        }, 500); // debounce 500ms
        return () => clearTimeout(timer);
    }, [amount, currency, limits]);

    async function handleBuy() {
        setLoading(true);
        try {
            const { signed_url } = await fetchApi<{ signed_url: string }>("/moonpay/sign-url", {
                method: "POST",
                body: { currency_code: currency },
                token,
            });
            // open MoonPay widget in popup
            window.open(signed_url, "moonpay", "width=480,height=700");
            setShowWidget(true);
        } finally {
            setLoading(false);
        }
    }

    async function copyAddress() {
        await navigator.clipboard.writeText(walletAddress);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    }

    return (
        <div style={{ maxWidth: 480, margin: "0 auto", padding: "24px 16px" }}>
            <h1>Add Funds</h1>

            {/* tabs */}
            <div style={{ display: "flex", gap: 8, marginBottom: 24 }}>
                <button
                    onClick={() => setTab("buy")}
                    style={{ fontWeight: tab === "buy" ? "bold" : "normal" }}
                >
                    Buy with Card
                </button>
                <button
                    onClick={() => setTab("transfer")}
                    style={{ fontWeight: tab === "transfer" ? "bold" : "normal" }}
                >
                    Send from Wallet
                </button>
            </div>

            {tab === "buy" && (
                <div>
                    {/* currency selector */}
                    <label>Token</label>
                    <select
                        value={currency}
                        onChange={e => setCurrency(e.target.value)}
                    >
                        <option value="sol">SOL</option>
                        <option value="usdc_sol">USDC</option>
                    </select>

                    {/* amount input */}
                    <label>Amount (USD)</label>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <span>$</span>
                        <input
                            type="number"
                            value={amount}
                            onChange={e => setAmount(e.target.value)}
                            min={limits?.base_min_amount}
                            max={limits?.base_max_amount}
                        />
                    </div>

                    {limits && (
                        <p style={{ fontSize: 12, color: "#888" }}>
                            Min ${limits.base_min_amount} — Max ${limits.base_max_amount}
                        </p>
                    )}

                    {/* quote preview */}
                    {quote && (
                        <div style={{ background: "#111", borderRadius: 8, padding: 16, margin: "16px 0" }}>
                            <div style={{ display: "flex", justifyContent: "space-between" }}>
                                <span>You get</span>
                                <span>{quote.quote_currency_amount.toFixed(4)} {currency.toUpperCase()}</span>
                            </div>
                            <div style={{ display: "flex", justifyContent: "space-between" }}>
                                <span>Fee</span>
                                <span>${quote.fee_amount.toFixed(2)}</span>
                            </div>
                            <div style={{ display: "flex", justifyContent: "space-between", fontWeight: "bold" }}>
                                <span>Total charged</span>
                                <span>${quote.total_amount.toFixed(2)}</span>
                            </div>
                        </div>
                    )}

                    {/* wallet address preview */}
                    <p style={{ fontSize: 12, color: "#888" }}>
                        Funds will be sent to your wallet:
                    </p>
                    <p style={{ fontFamily: "monospace", fontSize: 11, wordBreak: "break-all" }}>
                        {walletAddress}
                    </p>

                    <button
                        onClick={handleBuy}
                        disabled={loading || !quote}
                        style={{ width: "100%", padding: "14px", marginTop: 16 }}
                    >
                        {loading ? "Opening MoonPay..." : `Buy ${currency.toUpperCase()} with Card`}
                    </button>

                    {showWidget && (
                        <p style={{ textAlign: "center", color: "#888", marginTop: 12 }}>
                            Complete your purchase in the MoonPay window.
                            Your balance will update automatically when done.
                        </p>
                    )}
                </div>
            )}

            {tab === "transfer" && (
                <div style={{ textAlign: "center" }}>
                    <p>Send SOL or USDC from any exchange or wallet to your address below</p>

                    <div style={{ margin: "24px auto", display: "inline-block" }}>
                        <QRCodeSVG
                            value={walletAddress}
                            size={200}
                            bgColor="#000000"
                            fgColor="#ffffff"
                        />
                    </div>

                    <p style={{ fontFamily: "monospace", fontSize: 12, wordBreak: "break-all", margin: "12px 0" }}>
                        {walletAddress}
                    </p>

                    <button onClick={copyAddress} style={{ width: "100%", padding: 12 }}>
                        {copied ? "✓ Copied!" : "Copy Address"}
                    </button>

                    <p style={{ color: "#f90", fontSize: 12, marginTop: 16 }}>
                        ⚠ Only send on Solana network. Sending on other networks will result in permanent loss.
                    </p>
                </div>
            )}
        </div>
    );
}