import { cors } from 'hono/cors';

export const corsMiddleware = cors({
  origin: (origin) => {
    const allowedOrigins = [
      'http://localhost:5173',
      'http://localhost:83',
      'https://repoxide.com',
      'https://api.repoxide.com',
    ];

    if (!origin || allowedOrigins.includes(origin)) {
      return origin;
    }

    if (origin.endsWith('.repoxide.pages.dev')) {
      return origin;
    }

    return null;
  },
  allowMethods: ['GET', 'POST', 'OPTIONS'],
  allowHeaders: ['Content-Type'],
  maxAge: 86400,
  credentials: true,
});
