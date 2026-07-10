import { handler } from "./build/handler.js";
import express from "express";
import proxy from "express-http-proxy";

const app = express();
const port = 3000;
const backend = process.env.BACKEND_URL || "http://localhost:13252";

app.use(
  "/api",
  proxy(backend, {
    proxyReqPathResolver: (req) => {
      const parts = req.url.split("?");
      const queryString = parts[1];
      let originalPath = parts[0];
      if (!originalPath.startsWith("/")) {
        originalPath = "/" + originalPath;
      }
      const updatedPath = `/api${originalPath}`;
      return updatedPath + (queryString ? "?" + queryString : "");
    },
  }),
);

app.use(handler);

app.listen(port, () => {
  console.log(`Listening on port ${port}`);
  console.log(`Proxy forwarding to ${backend}`);
});
