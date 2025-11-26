import type { ApiResult } from "./types";

export interface VikaClientOptions {
  baseUrl?: string;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  headers?: Record<string, string>;
  auth?: "bearerToken" | "fixed" | "consumerInjected";
}

export interface RequestContext {
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: any;
  rawOptions?: any;
}

export interface ResponseContext {
  request: RequestContext;
  response: Response;
  data: any;
}

type BeforeRequestMiddleware = (ctx: RequestContext) => Promise<void> | void;
type AfterResponseMiddleware = (ctx: ResponseContext) => Promise<void> | void;
type ErrorMiddleware = (error: any, ctx: RequestContext) => Promise<void> | void;

export interface RequestOptions {
  headers?: Record<string, string>;
  body?: any;
  signal?: AbortSignal;
  [key: string]: any;
}

export class VikaClient {
  private baseUrl: string;
  private timeout: number;
  private retries: number;
  private retryDelay: number;
  private headers: Record<string, string>;
  private auth?: "bearerToken" | "fixed" | "consumerInjected";
  private beforeRequest: BeforeRequestMiddleware[];
  private afterResponse: AfterResponseMiddleware[];
  private onError: ErrorMiddleware[];

  constructor(options: VikaClientOptions = {}) {
    this.baseUrl = options.baseUrl ?? "";
    this.timeout = options.timeout ?? 10000;
    this.retries = options.retries ?? 0;
    this.retryDelay = options.retryDelay ?? 250;
    this.headers = options.headers ?? {};
    this.auth = options.auth;
    this.beforeRequest = [];
    this.afterResponse = [];
    this.onError = [];
  }

  useBeforeRequest(fn: BeforeRequestMiddleware): void {
    this.beforeRequest.push(fn);
  }

  useAfterResponse(fn: AfterResponseMiddleware): void {
    this.afterResponse.push(fn);
  }

  useError(fn: ErrorMiddleware): void {
    this.onError.push(fn);
  }

  async request<
    SuccessMap extends Record<number, any>,
    ErrorMap extends Record<number, any>
  >(
    method: string,
    path: string,
    opts: RequestOptions = {}
  ): Promise<ApiResult<SuccessMap, ErrorMap>> {
    const url = path.startsWith("http") ? path : `${this.baseUrl}${path}`;
    
    // Prepare request context
    const requestContext: RequestContext = {
      method,
      url,
      headers: { ...this.headers, ...opts.headers },
      body: opts.body,
      rawOptions: opts,
    };

    // Run beforeRequest middlewares
    for (const middleware of this.beforeRequest) {
      await middleware(requestContext);
    }

    // Apply auth headers
    if (this.auth === "bearerToken") {
      // Consumer should inject token via middleware
      // This is a placeholder - actual implementation depends on consumer
    } else if (this.auth === "fixed") {
      // Consumer should set fixed headers via constructor
    }

    // Prepare fetch options
    const fetchOptions: RequestInit = {
      method,
      headers: requestContext.headers,
      signal: opts.signal,
    };

    if (opts.body !== undefined) {
      if (typeof opts.body === "string") {
        fetchOptions.body = opts.body;
      } else {
        fetchOptions.body = JSON.stringify(opts.body);
        if (!requestContext.headers["Content-Type"]) {
          requestContext.headers["Content-Type"] = "application/json";
        }
      }
    }

    // Retry logic
    let lastError: any;
    for (let attempt = 0; attempt <= this.retries; attempt++) {
      try {
        // Create abort controller for timeout
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), this.timeout);
        
        // Merge abort signal
        if (opts.signal) {
          opts.signal.addEventListener("abort", () => controller.abort());
        }

        const signal = controller.signal;
        fetchOptions.signal = signal;

        const response = await fetch(requestContext.url, fetchOptions);
        clearTimeout(timer);

        // Parse response
        let data: any;
        const contentType = response.headers.get("content-type");
        if (contentType && contentType.includes("application/json")) {
          try {
            data = await response.json();
          } catch {
            data = await response.text();
          }
        } else {
          data = await response.text();
        }

        // Prepare response context
        const responseContext: ResponseContext = {
          request: requestContext,
          response,
          data,
        };

        // Run afterResponse middlewares
        for (const middleware of this.afterResponse) {
          await middleware(responseContext);
        }

        // Determine if status is in success or error map
        const status = response.status as number;

        // Check if status is in success map (200-299)
        if (status >= 200 && status < 300) {
          return {
            ok: true,
            status: status as keyof SuccessMap,
            data: data as SuccessMap[keyof SuccessMap],
          } as ApiResult<SuccessMap, ErrorMap>;
        } else {
          // Status is in error map
          return {
            ok: false,
            status: status as keyof ErrorMap,
            error: data as ErrorMap[keyof ErrorMap],
          } as ApiResult<SuccessMap, ErrorMap>;
        }
      } catch (error: any) {
        lastError = error;

        // Check if we should retry
        const shouldRetry = 
          attempt < this.retries &&
          (error.name === "AbortError" ||
           error.message?.includes("network") ||
           error.message?.includes("fetch"));

        if (!shouldRetry) {
          // Run error middlewares
          for (const middleware of this.onError) {
            await middleware(error, requestContext);
          }
          throw error;
        }

        // Wait before retry
        await new Promise((resolve) => setTimeout(resolve, this.retryDelay * (attempt + 1)));
      }
    }

    // All retries exhausted
    for (const middleware of this.onError) {
      await middleware(lastError, requestContext);
    }
    throw lastError;
  }

  async get<SuccessMap extends Record<number, any>, ErrorMap extends Record<number, any>>(
    path: string,
    opts: RequestOptions = {}
  ): Promise<ApiResult<SuccessMap, ErrorMap>> {
    return this.request<SuccessMap, ErrorMap>("GET", path, opts);
  }

  async post<SuccessMap extends Record<number, any>, ErrorMap extends Record<number, any>>(
    path: string,
    opts: RequestOptions = {}
  ): Promise<ApiResult<SuccessMap, ErrorMap>> {
    return this.request<SuccessMap, ErrorMap>("POST", path, opts);
  }

  async put<SuccessMap extends Record<number, any>, ErrorMap extends Record<number, any>>(
    path: string,
    opts: RequestOptions = {}
  ): Promise<ApiResult<SuccessMap, ErrorMap>> {
    return this.request<SuccessMap, ErrorMap>("PUT", path, opts);
  }

  async patch<SuccessMap extends Record<number, any>, ErrorMap extends Record<number, any>>(
    path: string,
    opts: RequestOptions = {}
  ): Promise<ApiResult<SuccessMap, ErrorMap>> {
    return this.request<SuccessMap, ErrorMap>("PATCH", path, opts);
  }

  async delete<SuccessMap extends Record<number, any>, ErrorMap extends Record<number, any>>(
    path: string,
    opts: RequestOptions = {}
  ): Promise<ApiResult<SuccessMap, ErrorMap>> {
    return this.request<SuccessMap, ErrorMap>("DELETE", path, opts);
  }
}

// Default instance - consumers can configure this or create their own
export const vikaClient = new VikaClient();

