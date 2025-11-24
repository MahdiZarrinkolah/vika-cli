const requestInitIndicators = [
  "method",
  "headers",
  "body",
  "signal",
  "credentials",
  "cache",
  "redirect",
  "referrer",
  "referrerPolicy",
  "integrity",
  "keepalive",
  "mode",
  "priority",
  "window",
];

const isRequestInitLike = (value: unknown): value is RequestInit => {
  if (!value || typeof value !== "object") {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return requestInitIndicators.some((key) => key in candidate);
};

export const http = {
  // GET helper. Second argument can be either a RequestInit or a JSON body for uncommon GET-with-body endpoints.
  async get<T = any>(url: string, optionsOrBody?: RequestInit | unknown): Promise<T> {
    let init: RequestInit = { method: "GET", body: null };

    if (optionsOrBody !== undefined && optionsOrBody !== null) {
      if (isRequestInitLike(optionsOrBody)) {
        const candidate = optionsOrBody as RequestInit;
        init = {
          ...candidate,
          method: "GET",
          body: candidate.body ?? null,
        };
      } else {
        init = {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(optionsOrBody),
        };
      }
    }

    const response = await fetch(url, {
      ...init,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async post<T = any>(url: string, body?: any, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
      body: body !== undefined ? JSON.stringify(body) : options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async put<T = any>(url: string, body?: any, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
      body: body !== undefined ? JSON.stringify(body) : options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async delete<T = any>(url: string, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "DELETE",
      body: options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async patch<T = any>(url: string, body?: any, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
      body: body !== undefined ? JSON.stringify(body) : options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async head(url: string, options: RequestInit = {}): Promise<Response> {
    const response = await fetch(url, {
      ...options,
      method: "HEAD",
      body: options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response;
  },

  async options<T = any>(url: string, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "OPTIONS",
      body: options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },
};


