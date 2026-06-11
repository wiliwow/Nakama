# Troubleshooting Guide: Ollama Windows Port 11434 Bind Error

```
Error: listen tcp 127.0.0.1:11434: bind: 
Only one usage of each socket address (protocol/network address/port) is normally permitted.
```

---

## What This Error Means

Ollama's API server listens on `127.0.0.1:11434` by default. This error means **something else is already bound to that port**. Because TCP allows only one listening socket per `(protocol, IP, port)` tuple, the second listener fails with `EADDRINUSE` — exactly what Windows surfaces as the message above.

There are three scenarios this covers:

| Scenario | Cause |
|---|---|
| Stale Ollama daemon already running from a previous session | Most common — Ollama is designed to run headless in the background; it does not stop when you close the terminal that launched it |
| Another application also listening on 11434 | Possible if another tool or service was configured for the same port |
| Port is in `TIME_WAIT` from a recently crashed process | The OS hasn't released the socket yet; resolves itself after ~60 s or can be forced |

---

## Step 1 — Identify the Process Using Port 11434

### Method 1: netsh (most reliable on Windows 10/11)

Run this in **PowerShell as Administrator** or a standard command prompt:

```powershell
netsh interface ipv4 show excludedportrange protocol=tcp
```

This lists reserved port ranges by the OS. If 11434 lands inside an excluded range the OS is blocking you.

### Method 2: findstr via netstat (identifies the exact process)

Run this PowerShell one-liner:

```powershell
netstat -ano | Select-String ":11434\s+LISTENING"
```

**Sample output and how to read it:**
```
  TCP    127.0.0.1:11434        0.0.0.0:0              LISTENING       8240
```
The last column (`8240`) is the **PID** (Process ID) of the process holding the port.

### Method 3: GetOwningProcess via PowerShell (shows process name directly)

```powershell
@(Get-NetTCPConnection -LocalPort 11434 -ErrorAction SilentlyContinue |
    Select-Object -ExpandProperty OwningProcess)
```

### Method 4: Task Manager lookup

Take the PID from Method 2 and cross-reference it:

1. Open **Task Manager** (`Ctrl+Shift+Esc`)
2. Go to the **Details** tab
3. Enable the **PID** column (right-click column header → Select columns → check **PID**)
4. Locate the matching PID — the **Name** column tells you which process holds the port

---

## Step 2 — Resolve the Conflict

### Option A: If the process IS Ollama (PID belongs to `ollama.exe`) → **stop it gracefully**

Ollama is already running headlessly in the background. You don't need `ollama serve` — it is redundant. Just verify it is reachable:

```powershell
# Verify Ollama is working on the expected port
curl http://127.0.0.1:11434
```

If you get a JSON response Ollama is healthy. Use it directly from your application without starting a new server.

**Why this happens:** Ollama sets itself up as a persistent background service installed in `%LocalAppData%\Programs\Ollama`. Every time you double-click the Ollama Desktop icon or run `ollama serve`, another daemon instance tries to start on the same port and fails.

---

### Option B: If the process is NOT Ollama → **terminate it**

**1. Kill by PID** (replace `8240` with your actual PID):

```powershell
taskkill /PID 8240 /F
```

**2. Force-kill by image name** (use if PID isn't available):

```powershell
taskkill /IM processname.exe /F
```

For example:
```powershell
taskkill /IM ollama.exe /F        # kills all Ollama processes
taskkill /IM node.exe /F          # kills Node.js (use with care — verify the PID first!)
```

**3. Confirm the port is free before trying again:**

```powershell
netstat -ano | Select-String ":11434\s+LISTENING"
```

If there is no output, the port is free.

---

## Step 3 — Start Ollama Correctly After Freeing the Port

### Scenario A: You want a background daemon (recommended)

Do **not** run `ollama serve` manually. Use the installed service instead:

```powershell
# Ensure the Ollama service is running and set to start automatically
ollama serve
```

This starts the daemon in the current terminal. Close the terminal and Ollama keeps running.

### Scenario B: You want it as a Windows Service (most robust)

```powershell
# Admin PowerShell — creates a persistent background service
sc create Ollama binPath= "C:\Users\wiliwow\AppData\Local\Programs\Ollama\ollama.exe serve" start= auto
sc start Ollama
```

### Scenario C: Dev environment where you need to restart cleanly

```powershell
# 1. Kill any existing Ollama process
taskkill /IM ollama.exe /F

# 2. Wait 3 seconds for the OS to release the socket
Start-Sleep -Seconds 3

# 3. Start fresh
ollama serve
```

---

## Step 4 — Prevent the Error in Future Sessions

### Prevention checklist

| Action | How |
|---|---|
| Check Ollama is already running before `serve` | `Get-Process ollama -ErrorAction SilentlyContinue` |
| Use the same start method every time | Pick either Service, Desktop app, or `ollama serve` — never mix |
| Increase OS ephemeral port range if hitting `TIME_WAIT` bothered frequently | `netsh int ipv4 set dynamicport tcp start=10000 num=55535` |
| Allow Ollama through Windows Defender Firewall | Add inbound rule for `ollama.exe` on port 11434 |

### Quick preflight check one-liner

Drop this in a script at build/test time to fail fast with a readable message:

```powershell
$port = netstat -ano | Select-String ":11434\s+LISTENING"
if ($port) {
    Write-Error "Port 11434 is already in use by PID $($port -replace '.*\s+(\d+)\s*$','$1'). Stop the existing process before running 'ollama serve'."
    exit 1
}
ollama serve
```

---

## Quick Reference: Common Commands

| Goal | Command |
|---|---|
| Find process on port 11434 | `netstat -ano \| Select-String ":11434.+"` |
| Kill process by PID | `taskkill /PID <pid> /F` |
| Kill all Ollama processes | `taskkill /IM ollama.exe /F` |
| Verify Ollama responds | `curl http://127.0.0.1:11434` |
| Start Ollama daemon | `ollama serve` |
| Check TIME_WAIT sockets on a port | `netstat -ano -p tcp` |

---

## Why Running `ollama serve` Twice Fails Specifically on Windows

- **Windows does not `SO_REUSEADDR` by default** on loopback listeners in the same process. A new `ollama serve` attempts `bind()` on the exact same `(127.0.0.1, 11434)` tuple.
- **Linux/macOS would allow it** if the first socket had `SO_REUSEADDR` set. Windows requires the old listener to be explicitly closed first — hence the `EADDRINUSE` error.
- **Ollama Desktop installs a resident agent** at `%LocalAppData%\Programs\Ollama\ollama.exe`. It runs from startup and stays resident. Running `ollama serve` on top of it is always redundant on Windows.

---

*If the error persists after confirming the port is free and no Ollama process is running, a leftover Windows TCP filter or third-party VPN/tunneling tool may be holding the port quietly. Disable competing network tools and retry.*
