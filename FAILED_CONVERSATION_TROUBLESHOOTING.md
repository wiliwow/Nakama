# Troubleshooting Guide: "Failed to Create Conversation" Error

## Overview

The "Failed to create conversation" error occurs when the client application cannot establish a new chat session with the AI agent backend. This guide organizes potential root causes into four categories, each with diagnostic procedures and actionable remediation steps.

---

## 1. API Connectivity and Network Latency

### Error Analysis

This category covers scenarios where the API request never reaches the server — or reaches it so degraded that the backend times out before constructing the conversation object. Common triggers include DNS resolution failures, TCP connection resets, high RTT causing gateway timeouts, or an unreachable base URL due to a misconfigured environment variable or proxy.

### Diagnostic Procedures

| Step | Action |
|---|---|
| 1. Check DNS | `nslookup <api-base-url>` or `dig <api-base-url>` — confirms the hostname resolves to an IP |
| 2. Test raw connectivity | `curl -v https://<api-base-url>/health` — inspects TLS handshake and HTTP response |
| 3. Inspect latency | `curl -w "@curl-format.txt" -o /dev/null -s https://<api-base-url>/health` — measure RTT in milliseconds |
| 4. Review proxy config | Check `HTTP_PROXY`, `HTTPS_PROXY`, and `NO_PROXY` env vars; invalid proxy values silently route or block traffic |
| 5. Inspect client network logs | In DevTools → Network tab, filter by the conversation-creation endpoint; look for `(failed)`, `ERR_NAME_NOT_RESOLVED`, `ERR_CONNECTION_REFUSED`, or `ERR_CONNECTION_RESET` |
| 6. Check firewall / egress rules | Confirm outbound port 443 (HTTPS) is not restricted by host-level firewalls, corporate proxies, or cloud security groups (e.g., Azure NSG, AWS SGB) |

### Debugging Steps

1. **Verify the base URL** — Ensure the API base URL in the client configuration (env file, `.env`, or secrets manager) matches the active deployment. A production URL pointed at a staging address, or vice-versa, will silently fail at the network layer.

2. **Reproduce outside the application** — Use `curl` or `Postman` with the exact same headers and body the application sends. If the tooling also fails, the problem is network-level, not application-level.

3. **Test with a different network** — Switch from corporate Wi-Fi to a hotspot or VPN. If the error clears, a corporate proxy or internal DNS is the cause.

4. **Check for TLS inspection** — Corporate man-in-the-middle TLS proxies (e.g., Zscaler, Palo Alto, Symantec) can cause certificate validation failures that surface as generic connection errors. Ensure the CA bundle used by the application trusts the corporate root CA, or disable TLS verification temporarily (only for testing) to confirm the hypothesis.

5. **Enable verbose network logging on the client** — most HTTP libraries support a debug/trace log level. Enable it and review the full request-response lifecycle, including redirects and headers.

### Remediation

- Fix the base URL in environment configuration.
- Add the API host to proxy bypass rules (`NO_PROXY`).
- Work with network/security teams to whitelist the API host on port 443.
- Update the CA certificate bundle if TLS inspection is in place.

---

## 2. Authentication and Authorization Failures

### Error Analysis

Conversation creation endpoints almost universally require a valid, active credential. An expired JWT, a revoked API key, a malformed `Authorization` header, or a missing credential will cause the backend to reject the request before any conversation state is created. Because the frontend often surfaces all non-2xx responses through the same UI path — a red pop-up — the generic "Failed to create conversation" banner masks the specific HTTP status code.

### Diagnostic Procedures

| Step | Action |
|---|---|
| 1. Inspect HTTP status code | In DevTools → Network tab, click the failed request and read `Status`. `401` = unauthenticated; `403` = authenticated but unauthorized |
| 2. Check response body | The server usually returns a JSON body with `error`, `message`, or `code` fields describing the reason — review this, not just the UI banner |
| 3. Inspect the `Authorization` header | In DevTools → Network → Request Headers, confirm `Authorization: Bearer <token>` or `x-api-key: <key>` is present and correctly formatted |
| 4. Decode the JWT | Use [jwt.io](https://jwt.io) to check the `exp` claim. If `exp` is in the past, the token is expired |
| 5. Check token scope/permissions | For OAuth or scoped API keys, verify the token includes the scope required for `conversations:create` or `chat:write` |
| 6. Audit token rotation | If the app uses short-lived tokens (common pattern: refresh every 15 min), check the refresh-token flow in browser/server logs for failures |

### Debugging Steps

1. **Capture the full request and response headers** — compare against a known-good request (e.g., a previously successful conversation) to identify what is missing or changed.

2. **Validate the API key in a tool** — paste the key directly into Postman or curl. If Postman also returns 401/403, the key is invalid or its scope is wrong.

3. **Check the secrets store** — If the application reads credentials from a vault (Vault, AWS Secrets Manager, Doppler, etc.), confirm the secret version is current and the secret path/label is correct in the deployment config.

4. **Review the token generation pipeline** — For JWT-based auth, confirm the signing key on the token issuer matches the verification key on the server. A key rotation that was not propagated to all services will cause silent auth failures.

5. **Test with a newly generated credential** — Generate a fresh API key and retest. This eliminates the possibility that the stored credential has drifted without updating configuration.

6. **Check for account-level blocks** — Some platforms suspend or rate-limit individual accounts rather than returning a generic error. Check the account portal or billing dashboard for suspension flags.

### Remediation

- Reissue or rotate the expired/invalid token or API key.
- Update the secret in the vault or environment variable store, then restart affected services.
- Expand the token/permission scope to include `conversations:create` if it was omitted.
- Implement proactive token refresh — ideally refresh 60 seconds before `exp` to prevent edge-case race conditions.
- Enable structured logging on the auth middleware to capture the exact `reason` code the server returns on failure.

---

## 3. Rate Limiting and Quota Exhaustion

### Error Analysis

Most AI/LLM APIs enforce rate limits (requests per minute/hour) and hard quotas (monthly token usage or API call budgets). When a client exceeds either threshold, the backend responds with a `429 Too Many Requests` or a `403/402 Payment Required` (quota exhausted) status. Because the client may not surface the status code to the user, it appears as the same generic "Failed to create conversation" red pop-up.

### Diagnostic Procedures

| Step | Action |
|---|---|
| 1. Check the response status code | `429` = rate limited. `402` / `403` (with billing/insufficient_credits reason) = quota exhausted. |
| 2. Inspect `Retry-After` header | On a 429 response, the server usually includes `Retry-After: <seconds>` — tells you how long to wait before the next attempt succeeds |
| 3. Review `X-RateLimit-*` headers | `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers reveal the current ceiling and window |
| 4. Check billing/usage dashboard | Log into the provider portal (OpenAI, Anthropic, etc.) and review usage graphs for the current billing period |
| 5. Inspect client-side request queue | Confirm the application is not rapidly retrying failed requests without backoff, which compounds the problem |
| 6. Review server access logs | Look for repeated 429 responses from the same IP/client ID in a short window to confirm exhaustion |

### Debugging Steps

1. **Enable response header inspection** — In DevTools or the client's HTTP debug logger, record all response headers on each request. Confirm whether rate-limit headers are being returned and whether remaining quota is approaching zero.

2. **Implement an exponential backoff with jitter** — If the client has a retry loop, confirm it respects `Retry-After` and backs off exponentially rather than hammering the endpoint.

3. **Check for burst traffic patterns** — Initiate a debounced request queue or a token-bucket limiter on the client side before requests hit the server.

4. **Review concurrent request count** — Some APIs allow only one active conversation per account or per token. If the application fires multiple conversation-creation calls in parallel, each may be rejected.

5. **Audit the usage quota** — For paid tiers, check whether the provider's monthly token or API call cap is near its limit. For free tiers, check whether the free tier's hard cap has been reached.

6. **Monitor server-side metrics** — If you control the backend, check rate-limit counters (e.g., Redis sliding-window counters, Nginx `limit_req`, or API gateway rate-limit stats) to confirm the limit threshold is being reached.

### Remediation

- Implement client-side rate limiting with a token bucket or leaky bucket algorithm.
- Cache or reuse existing conversations rather than creating a new one per prompt.
- Increase the API quota tier or upgrade the plan.
- Add a `429` handler on the client: surface a user-facing message that says "Too many requests — please wait and try again" instead of the generic error.
- Contact the API provider to request a temporary quota increase if the spike was unexpected.

---

## 4. Server-Side Errors and Backend Infrastructure Issues

### Error Analysis

When the server itself experiences an internal failure — unhandled exception, database write failure, crashed worker, misconfigured middleware, or infrastructure outage — conversation creation fails with an HTTP `500`, `502`, `503`, or `504`. These errors indicate that the problem is on the server side, and the client cannot resolve it through reconfiguration alone. The error still manifests identically in the UI due to generic error handling.

### Diagnostic Procedures

| Step | Action |
|---|---|
| 1. Check the HTTP status code | `5xx` server errors confirm a backend failure. `502` = bad gateway, `503` = service unavailable, `504` = gateway timeout |
| 2. Review response body | Server error responses often include `error_id`, `trace_id`, or `request_id` — capture this for correlation with backend logs |
| 3. Check provider status page | Visit the provider's status page (e.g., status.openai.com, status.anthropic.com) for ongoing incidents |
| 4. Correlate with backend logs | Use the `request_id` or `trace_id` from the response to look up the exact log line on the server |
| 5. Review application server logs | Search for `ERROR` or `CRITICAL` entries at the timestamp of the failed request; look for stack traces around conversation creation |
| 6. Check database connectivity | If conversation creation writes to a database, confirm DB connectivity, connection pool health, and write latency |
| 7. Inspect infrastructure metrics | Check CPU, memory, disk usage, and pod/container restart counts on the backend service |

### Debugging Steps

1. **Correlate the `request_id` across systems** — Use the request/trace ID from the client error response to find the exact request in the server logs. If the ID is absent from server logs, the request may have been dropped before reaching the application layer (e.g., API gateway issue).

2. **Review server log stack traces** — Look specifically for exceptions in the conversation creation handler: null pointer errors, database constraint violations, serialization failures, or unhandled middleware exceptions.

3. **Check database write health** — If the conversation creation involves a DB INSERT, confirm the database is healthy and reachable. A failed connection pool or a full write lock in the DB (e.g., MariaDB lock wait timeout, PostgreSQL deadlock) will surface as a `500`.

4. **Review the infrastructure health dashboard** — High CPU/RAM usage on backend pods, a cloud provider outage, a misbehaving dependent service (e.g., embedding model, vector DB, payment service) will all surface as `500` errors on conversation creation.

5. **Check the casing and shape of the request payload** — A malformed or schema-invalid request body that passes client-side validation can cause the backend to throw a serialization error (e.g., a required field typed as `null` in a language that doesn't allow it at serialization time).

6. **Test with minimal payload** — Send the simplest possible valid conversation creation request (no metadata, no attachments, minimal fields) to determine whether a specific field or attachment in the payload triggers the error.

7. **Review deployment history** — If the error appeared after a recent deploy, check the deploy diff for changes to the conversation creation endpoint, database schema, or middleware stack.

### Remediation

- Deploy a patch for the underlying server-side bug; open a hotfix or roll back the recent deployment if it is the known cause.
- Add missing null-checks, schema validations, or database constraints that were tolerating invalid input before.
- Scale up backend resources or add auto-scaling rules if capacity is the limiting factor.
- Add circuit-breaker patterns around dependent services (embedding model, vector DB, payment provider) so that a downstream failure does not cascade as a generic conversation-error.
- Add structured error response envelopes (e.g., `{ error: { code, message, request_id } }`) so the client can surface the specific backend reason rather than a generic banner.

---

## Cross-Cutting: Escalation Decision Matrix

| Status Code | Likely Category | Can the client fix it alone? |
|---|---|---|
| `400` | Auth / bad payload | Partial — auth can be client-fixed; payload bugs may be server-side |
| `401` | Auth / expired/invalid token | Yes |
| `403` | Auth / scope or account block | Possibly — depends on whether it's a missing scope or platform-level block |
| `404` | Enzyme (wrong endpoint URL) | Yes |
| `429` | Rate limiting / quota | Partial — client can add backoff, but quota increase requires account action |
| `5xx` | Server-side / infrastructure | No — requires backend/infra team action |

---

## Recommended Client-Side Improvements

To move from the generic red pop-up to actionable user feedback:

1. **Expose the HTTP status code in the UI** — show `"Authentication failed (401)", "Rate limit exceeded (429)", "Server error (500)"` instead of a single message.
2. **Include the `request_id` in the error UI** — enables users to forward it to support for rapid triage.
3. **Add per-error-type user guidance** — e.g., "Check your API key" for 401, "Wait a moment" for 429, "Known issue, contact support" for 5xx.
4. **Log all error metadata server-side** — store `request_id`, status code, response body, and client IP for every failed conversation creation for post-mortem analysis.

---

*This guide covers the four most common root-cause categories for the "Failed to create conversation" error. Start with the HTTP status code from the browser/network tab to determine which section to follow.*
