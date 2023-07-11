import { getClient, Body, RequestOptions, Response, HttpOptions } from "@tauri-apps/api/http";

export type HttpInterceptorsRequest = (config: HttpOptions) => Promise<HttpOptions>;
export type HttpInterceptorsResponse = (response: Response<any>) => Promise<Response<any>>;

interface HttpInterceptors {
    request: HttpInterceptorsRequest;
    response: HttpInterceptorsResponse;
}

export type HTTPClientOptions = HttpOptions & {
    params?: Record<string, any>;
};

export type HTTPClientRequest = RequestOptions & {
    params?: Record<string, any>;
};

export class HTTPClient {
    static httpClient: HTTPClient;

    static transformBody(body: any) {
        if (typeof body === "object") {
            body = Body.json(body);
        } else if (typeof body === "string") {
            body = Body.text(body);
        } else if (body instanceof Uint8Array) {
            body = Body.bytes(body);
        } else {
            body = Body.form(body);
        }
        return body;
    }

    public static getHttpClient(): HTTPClient {
        if (!this.httpClient) {
            this.httpClient = new HTTPClient();
        }
        return this.httpClient;
    }

    private innerBaseUrl = "";

    // 拦截器
    private innerInterceptors: HttpInterceptors = {
        request: async (config: HttpOptions) => config,
        response: async (response: Response<any>) => response,
    };

    public set baseUrl(url: string) {
        this.innerBaseUrl = url;
    }

    public get baseUrl() {
        return this.innerBaseUrl;
    }

    public set interceptors(interceptors: Partial<HttpInterceptors>) {
        if (interceptors.request) interceptors.request = interceptors.request;

        if (interceptors.response) interceptors.response = interceptors.response;
    }

    public get interceptorsRequest() {
        return this.innerInterceptors.request;
    }

    public set interceptorsRequest(value) {
        this.innerInterceptors.request = value;
    }

    public get interceptorsResponse() {
        return this.innerInterceptors.response;
    }

    public set interceptorsResponse(value) {
        this.innerInterceptors.response = value;
    }

    public async request<T>(options: HTTPClientOptions): Promise<T> {
        const client = await getClient();

        const url = `${this.baseUrl}${options.url}` + (options.params ? `?${new URLSearchParams(options.params)}` : "");

        const queryInnerOptions = {
            ...options,
            url: url,
        } as HttpOptions;

        if (options.body) {
            queryInnerOptions.body = HTTPClient.transformBody(options.body);
        }

        const queryOptions = await this.interceptorsRequest(queryInnerOptions);

        const res = await client.request<T>(queryOptions);

        return (await this.interceptorsResponse(res)).data;
    }

    public async get<T>(url: string, options?: HTTPClientRequest): Promise<T> {
        return this.request<T>({
            url,
            ...options,
            method: "GET",
        });
    }

    public async post<T>(url: string, body?: any, options?: HTTPClientRequest): Promise<T> {
        return this.request<T>({
            url,
            body,
            ...options,
            method: "POST",
        });
    }

    public async put<T>(url: string, body?: any, options?: HTTPClientRequest): Promise<T> {
        return this.request<T>({
            url,
            body,
            ...options,
            method: "PUT",
        });
    }

    public async delete(url: string, options?: HTTPClientRequest) {
        return this.request({
            url,
            ...options,
            method: "DELETE",
        });
    }
}

export const http = HTTPClient.getHttpClient()
