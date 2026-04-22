const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "/api";

type RequestOptions = {
  method?: "GET" | "POST" | "PUT" | "DELETE";
  body?: any;
  token?: string | null;
};

export async function fetchApi<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, token } = options;

  const headers: Record<string, string> = {};
  if (body) {
    headers["Content-Type"] = "application/json";
  }
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    const errorData = await response.json().catch(() => null);
    throw new Error(errorData?.error || errorData?.message || `API request failed: ${response.statusText}`);
  }

  // Handle empty 204 responses or similar
  const text = await response.text();
  return text ? JSON.parse(text) : {};
}
