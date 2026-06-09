import { useAuthStore } from "@/store/useStore";

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "/api";

type RequestOptions = {
  method?: "GET" | "POST" | "PUT" | "DELETE";
  body?: any;
  token?: string | null;
};

export async function fetchApi<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, token } = options;
  const authToken = token ?? useAuthStore.getState().token;

  const headers: Record<string, string> = {};
  if (body) {
    headers["Content-Type"] = "application/json";
  }
  if (authToken) {
    headers["Authorization"] = `Bearer ${authToken}`;
  }

  // Inject current network header dynamically
  const network = useAuthStore.getState().network || "mainnet";
  headers["x-network"] = network;

  let response = await fetch(`${API_BASE_URL}${endpoint}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  if (response.status === 401 && endpoint !== "/auth/refresh") {
    const { refreshToken, setToken, logout } = useAuthStore.getState();
    if (refreshToken) {
      const refreshResponse = await fetch(`${API_BASE_URL}/auth/refresh`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: refreshToken }),
      });
      if (refreshResponse.ok) {
        const refreshed = await refreshResponse.json();
        setToken(refreshed.token);
        headers["Authorization"] = `Bearer ${refreshed.token}`;
        response = await fetch(`${API_BASE_URL}${endpoint}`, {
          method,
          headers,
          body: body ? JSON.stringify(body) : undefined,
        });
      } else {
        logout();
      }
    }
  }

  if (!response.ok) {
    const errorData = await response.json().catch(() => null);
    throw new Error(errorData?.error || errorData?.message || `API request failed: ${response.statusText}`);
  }

  const text = await response.text();
  return text ? JSON.parse(text) : ({} as T);
}
