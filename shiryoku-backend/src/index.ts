export interface Env {
  DB: D1Database;
  API_SECRET: string;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === '/pixel.png') {
      const id = url.searchParams.get('id') || 'unknown';

      const ip = request.headers.get('CF-Connecting-IP') || 'unknown';
      const country = request.headers.get('CF-IPCountry') || 'unknown';
      const city = request.cf?.city || 'unknown';
      const userAgent = request.headers.get('User-Agent') || 'unknown';
      const timestamp = new Date().toISOString();

      const query = `
				INSERT INTO logs (tracking_id, timestamp, ip, country, city, user_agent)
				VALUES (?, ?, ?, ?, ?, ?)
			`;

      ctx.waitUntil(
        env.DB.prepare(query)
          .bind(id, timestamp, ip, country, city, userAgent)
          .run()
          .catch((err) => console.error('DB Error', err)),
      );

      const gifData = [
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x21, 0xf9, 0x04,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x01, 0x44, 0x00, 0x3b,
      ];
      const gif = new Uint8Array(gifData);

      return new Response(gif, {
        headers: {
          'Content-Type': 'image/gif',
          'Cache-Control': 'private, no-cache, no-store, must-revalidate, max-age=0',
          Pragma: 'no-cache',
          Expires: 'Mon, 01 Jan 1990 00:00:00 GMT',
        },
      });
    }

    if (url.pathname === '/api/logs') {
      const secret = url.searchParams.get('secret');
      if (secret !== env.API_SECRET) {
        return new Response('Unauthorized', { status: 401 });
      }

      const { results } = await env.DB.prepare('SELECT * FROM logs ORDER BY id DESC LIMIT 100').all();

      return Response.json(results);
    }

    return new Response('Shiryoku Backend Active', { status: 200 });
  },
};
