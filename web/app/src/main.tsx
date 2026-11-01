import { QueryClientProvider } from '@tanstack/react-query';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { createBrowserRouter, createHashRouter, RouterProvider } from 'react-router';
import { queryClient } from '@/lib/queryClient';
import { routes } from '@/routes';
import '@/styles/global.css';

const isDesktop = typeof window !== 'undefined' && !!((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);

if (isDesktop) {
  const origLog = console.log;
  const origErr = console.error;
  const origWarn = console.warn;
  const send = (level: string, args: any[]) => {
    try {
      const internals = (window as any).__TAURI_INTERNALS__;
      if (internals?.invoke) {
        internals.invoke('log_from_js', {
          msg: `[${level}] ` + args.map((a) => {
            try {
              return typeof a === 'object' ? JSON.stringify(a) : String(a);
            } catch {
              return String(a);
            }
          }).join(' '),
        });
      }
    } catch {}
  };
  console.log = (...args) => { origLog(...args); send('LOG', args); };
  console.error = (...args) => { origErr(...args); send('ERR', args); };
  console.warn = (...args) => { origWarn(...args); send('WARN', args); };
  window.addEventListener('error', (e) => send('UNCAUGHT', [e.message, e.filename, e.lineno]));
  window.addEventListener('unhandledrejection', (e) => send('PROMISE', [e.reason]));
}

const router = isDesktop ? createHashRouter(routes) : createBrowserRouter(routes);

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
);
