# EPP Lab frontend

The frontend is a React/Vite application using TanStack Router, TanStack
Query, Radix UI, Tailwind CSS and browser `fetch`.

## Development

```bash
npm install
npm run dev
```

The Vite development server proxies `/api` to the local Axum API at
`http://localhost:8080`.

## Checks

```bash
npm run lint
npm run build
```

Production assets are written to `frontend/dist` and are served by the Axum
application in the production deployment stage.
