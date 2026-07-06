import type { Env } from './types';
import { errorResponse } from './shared/http';
import { routeRequest } from './routes';

export { ChatRealtimeObject } from './chat/realtime';

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      return await routeRequest(request, env);
    } catch (error) {
      return errorResponse(error);
    }
  }
};
