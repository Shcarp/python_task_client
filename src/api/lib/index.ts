import { HttpOptions } from "@tauri-apps/api/http";
import { http } from "../../utils/httpClient";

http.baseUrl = "http://127.0.0.1:3306";

http.interceptorsRequest = async (config: HttpOptions) => {
    config.headers = {
        Authorization: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6MSwidXNlcm5hbWUiOiJzdW51aWhAcXEuY29tIiwicGFzc3dvcmQiOiIxMjM0NTYiLCJlbWFpbCI6InN1bnVpaEBxcS5jb20iLCJwcm9maWxlX3BpY3R1cmUiOm51bGwsImxhc3RfYWN0aXZlIjoiMjAyMy0wNi0yOVQxNDo1MDozNC4wMDBaIiwiY3JlYXRlZF9hdCI6IjIwMjMtMDYtMjlUMTQ6NTA6MTQuMDAwWiIsInVwZGF0ZWRfYXQiOiIyMDIzLTA2LTI5VDE0OjUwOjE0LjAwMFoiLCJpYXQiOjE2ODg4MDI2MjZ9.RfdnMnqAecBNmCJSOuv4-1yi1Fh_Er1pcPhdswVyTFk"
    }
    return config;
};

export default http;
