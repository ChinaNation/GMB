// CPMS 前端通用类型：只放跨模块共享的 HTTP 与会话结构。

export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

export interface ApiError {
  code: number;
  error_code: string;
  message: string;
  trace_id: string;
}

export interface SessionUser {
  user_id: string;
  role: string;
  admin_name: string;
}
