// MoonPay configuration is loaded from environment variables at runtime.
// See: MOONPAY_PUBLISHABLE_KEY, MOONPAY_SECRET_KEY, MOONPAY_WEBHOOK_SECRET in .env
//
// The backend reads these via std::env::var() in handlers/moonpay_handlers.rs.
// No compile-time config struct is needed since this project uses env vars directly.
