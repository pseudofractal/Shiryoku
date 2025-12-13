import { connect } from 'cloudflare:sockets';

export interface Env {
  DB: D1Database;
  API_SECRET: string;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === '/pixel.png') {
      const id = url.searchParams.get('id') || 'unknown';
      const recentLog = await env.DB.prepare(`SELECT timestamp FROM logs WHERE tracking_id = ? ORDER BY id DESC LIMIT 1`).bind(id).first();

      let shouldLog = true;
      if (recentLog && recentLog.timestamp) {
        const lastTime = new Date(recentLog.timestamp as string).getTime();
        if (new Date().getTime() - lastTime < 60000) shouldLog = false;
      }

      if (shouldLog) {
        const ip = request.headers.get('CF-Connecting-IP') || 'unknown';
        const country = request.headers.get('CF-IPCountry') || 'unknown';
        const city = request.cf?.city || 'unknown';
        const userAgent = request.headers.get('User-Agent') || 'unknown';
        const timestamp = new Date().toISOString();
        const timezone = request.cf?.timezone || 'UTC';

        ctx.waitUntil(
          env.DB.prepare(
            `
            INSERT INTO logs (tracking_id, timestamp, ip, country, city, user_agent, timezone)
            VALUES (?, ?, ?, ?, ?, ?, ?)
          `,
          )
            .bind(id, timestamp, ip, country, city, userAgent, timezone)
            .run(),
        );
      }

      const gifData = [
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x21, 0xf9, 0x04,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x01, 0x44, 0x00, 0x3b,
      ];
      return new Response(new Uint8Array(gifData), {
        headers: { 'Content-Type': 'image/gif', 'Cache-Control': 'no-cache' },
      });
    }

    const secret = url.searchParams.get('secret');
    if (secret !== env.API_SECRET) return new Response('Unauthorized', { status: 401 });

    if (url.pathname === '/api/logs') {
      if (request.method === 'DELETE') {
        const trackingId = url.searchParams.get('tracking_id');
        if (!trackingId) return new Response('Missing tracking_id', { status: 400 });
        const result = await env.DB.prepare('DELETE FROM logs WHERE tracking_id = ?').bind(trackingId).run();
        return Response.json({ success: true, changes: result.meta.changes });
      }
      const { results } = await env.DB.prepare('SELECT * FROM logs ORDER BY id DESC LIMIT 100').all();
      return Response.json(results);
    }

    if (url.pathname === '/api/filters') {
      const recipients = await env.DB.prepare('SELECT DISTINCT tracking_id FROM logs').all();
      const countries = await env.DB.prepare('SELECT DISTINCT country FROM logs').all();
      return Response.json({
        recipients: recipients.results.map((r: any) => r.tracking_id),
        countries: countries.results.map((r: any) => r.country).filter((c: any) => c && c !== 'unknown'),
      });
    }

    if (url.pathname === '/api/schedule' && request.method === 'POST') {
      try {
        const formData = await request.formData();
        const recipient = formData.get('recipient') as string;
        const subject = formData.get('subject') as string;
        const html_body = formData.get('html_body') as string;
        const plain_body = formData.get('plain_body') as string;
        const scheduled_at = formData.get('scheduled_at') as string;
        const smtp_username = formData.get('smtp_username') as string;
        const smtp_password = formData.get('smtp_password') as string;
        const sender_name = formData.get('sender_name') as string;

        if (!recipient || !scheduled_at) return new Response('Missing fields', { status: 400 });

        const { results } = await env.DB.prepare(
          `
            INSERT INTO scheduled_emails (recipient, subject, html_body, plain_body, scheduled_at, smtp_username, smtp_password, sender_name)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
        `,
        )
          .bind(recipient, subject, html_body, plain_body, scheduled_at, smtp_username, smtp_password, sender_name)
          .run();

        const emailId = results[0].id;

        const processFiles = async (key: string, isInline: number) => {
          const files = formData.getAll(key);
          for (const entry of files) {
            if (entry instanceof File) {
              const base64 = await fileToBase64(entry);
              const cid = isInline ? entry.name : null;
              await env.DB.prepare(
                `
                INSERT INTO attachments (email_id, filename, content_type, data, is_inline, cid)
                VALUES (?, ?, ?, ?, ?, ?)
              `,
              )
                .bind(emailId, entry.name, entry.type, base64, isInline, cid)
                .run();
            }
          }
        };

        await processFiles('attachments', 0);
        await processFiles('inline_images', 1);

        return Response.json({ success: true, id: emailId });
      } catch (e: any) {
        return new Response(e.message, { status: 500 });
      }
    }

    return new Response('Not Found', { status: 404 });
  },

  async scheduled(_event: ScheduledEvent, env: Env, _ctx: ExecutionContext) {
    const now = new Date().toISOString();
    const { results } = await env.DB.prepare(
      `
      SELECT * FROM scheduled_emails
      WHERE status = 'pending' AND scheduled_at <= ?
    `,
    )
      .bind(now)
      .all();

    console.log(`Cron: Processing ${results.length} emails`);

    for (const email of results) {
      try {
        const attachments = await env.DB.prepare(`SELECT * FROM attachments WHERE email_id = ?`).bind(email.id).all();

        await sendSmtpEmail(email, attachments.results || []);

        await env.DB.prepare(`UPDATE scheduled_emails SET status = 'sent' WHERE id = ?`).bind(email.id).run();
        console.log(`Sent email ${email.id}`);
      } catch (e: any) {
        console.error(`Failed email ${email.id}:`, e.message);
        await env.DB.prepare(`UPDATE scheduled_emails SET status = 'failed' WHERE id = ?`).bind(email.id).run();
      }
    }
  },
};

async function fileToBase64(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  let binary = '';
  const bytes = new Uint8Array(buffer);
  for (let i = 0; i < bytes.byteLength; i++) binary += String.fromCharCode(bytes[i]);
  return btoa(binary);
}

// --- SMTP CLIENT ---
// Gemini is so good!

async function sendSmtpEmail(email: any, attachments: any[]) {
  const socket = connect('smtp.gmail.com:465', {
    secureTransport: 'on' as const,
    allowHalfOpen: false,
  });

  const writer = socket.writable.getWriter();
  const reader = socket.readable.getReader();
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  let buffer = '';

  // Helper to read SMTP responses
  const readUntilCode = async (code: string) => {
    while (true) {
      const idx = buffer.indexOf('\n');
      if (idx !== -1) {
        const line = buffer.slice(0, idx + 1);
        buffer = buffer.slice(idx + 1);
        const text = line.trim();
        if (text.startsWith(`${code} `)) return text;
        if (!text.startsWith(`${code}-`)) throw new Error(`SMTP Error: Expected ${code}, got: ${text}`);
        continue;
      }
      const { value, done } = await reader.read();
      if (done) throw new Error('Connection closed unexpectedly');
      buffer += decoder.decode(value, { stream: true });
    }
  };

  const writeCmd = async (cmd: string) => {
    await writer.write(encoder.encode(cmd + '\r\n'));
  };

  // Handshake
  await readUntilCode('220');
  await writeCmd('EHLO cloudflare-worker');
  await readUntilCode('250');

  // Auth
  await writeCmd('AUTH LOGIN');
  await readUntilCode('334');
  await writeCmd(btoa(email.smtp_username));
  await readUntilCode('334');
  await writeCmd(btoa(email.smtp_password));
  await readUntilCode('235');

  // Envelope
  await writeCmd(`MAIL FROM: <${email.smtp_username}>`);
  await readUntilCode('250');
  await writeCmd(`RCPT TO: <${email.recipient}>`);
  await readUntilCode('250');
  await writeCmd('DATA');
  await readUntilCode('354');

  // DATA STREAMING
  const boundary = `----=_Part_${Date.now()}`;
  const rawEmail = buildMimeMessage(email, attachments, boundary);

  // Stream the email body in 16KB chunks to prevent socket timeout/truncation
  const chunkSize = 16384;
  for (let i = 0; i < rawEmail.length; i += chunkSize) {
    const chunk = rawEmail.substring(i, i + chunkSize);
    await writer.write(encoder.encode(chunk));
  }

  // End the message strictly
  await writeCmd('.');
  await readUntilCode('250');

  // Cleanup
  await writeCmd('QUIT');
  await writer.close();
}

function buildMimeMessage(email: any, attachments: any[], boundary: string): string {
  const crlf = '\r\n';

  // Sanitize Body Text
  const sanitize = (text: string) => (text || '').replace(/\r?\n/g, crlf).replace(/^\./gm, '..');

  let msg = '';

  // Headers
  let fromHeader = email.smtp_username;
  if (email.sender_name) {
    fromHeader = `"${email.sender_name}" <${email.smtp_username}>`;
  }
  msg += `From: ${fromHeader}${crlf}`;
  msg += `To: ${email.recipient}${crlf}`;
  msg += `Subject: ${email.subject}${crlf}`;
  msg += `MIME-Version: 1.0${crlf}`;
  msg += `Content-Type: multipart/mixed; boundary="${boundary}"${crlf}${crlf}`;

  // Body Section (Multipart/Alternative)
  const altBoundary = `alt_${boundary}`;
  msg += `--${boundary}${crlf}`;
  msg += `Content-Type: multipart/alternative; boundary="${altBoundary}"${crlf}${crlf}`;

  // Plain Text
  msg += `--${altBoundary}${crlf}`;
  msg += `Content-Type: text/plain; charset=utf-8${crlf}`;
  msg += `Content-Transfer-Encoding: 7bit${crlf}${crlf}`;
  msg += `${sanitize(email.plain_body)}${crlf}${crlf}`;

  // HTML
  msg += `--${altBoundary}${crlf}`;
  msg += `Content-Type: text/html; charset=utf-8${crlf}`;
  msg += `Content-Transfer-Encoding: 7bit${crlf}${crlf}`;
  msg += `${sanitize(email.html_body)}${crlf}${crlf}`;

  msg += `--${altBoundary}--${crlf}`; // End of Body Part

  // Attachments
  for (const att of attachments) {
    msg += `--${boundary}${crlf}`;
    msg += `Content-Type: ${att.content_type || 'application/octet-stream'}; name="${att.filename}"${crlf}`;
    msg += `Content-Transfer-Encoding: base64${crlf}`;

    if (att.is_inline && att.cid) {
      msg += `Content-ID: <${att.cid}>${crlf}`;
      msg += `Content-Disposition: inline; filename="${att.filename}"${crlf}`;
    } else {
      msg += `Content-Disposition: attachment; filename="${att.filename}"${crlf}`;
    }

    msg += `${crlf}`;

    // Insert Base64 Data
    const raw = att.data;
    for (let i = 0; i < raw.length; i += 76) {
      msg += raw.substring(i, i + 76) + crlf;
    }
  }

  // Final Footer
  msg += `--${boundary}--${crlf}`;

  return msg;
}
