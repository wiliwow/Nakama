# Nakama Long-Term Memory - Quick Reference

## Frontend: React Hooks & Components

### IndexManager Component
```typescript
import IndexManager from "./components/IndexManager";

// In App.tsx or parent component
<IndexManager />  // Renders with 📚 button (top-right)
```

**Features**:
- System health check (embeddings + LLM)
- File upload & indexing
- Index statistics display
- Clear index functionality

### ChatContainer Integration
```typescript
// Files are auto-indexed when message is sent
// Staged files appear in UI with "Index Files" button

// Auto-retrieval on query
const retrieved = await invoke<RetrievedPassage[]>(
    "rag_retrieve",
    { query_text: userInput, top_k: 10 }
);
```

## Backend: Tauri Commands

### Core Commands

**Index File**
```rust
invoke("rag_add_file", {
    filename: "document.txt",
    content: "file content here..."
})
→ Promise<string>  // "file 'document.txt' indexed"
```

**Retrieve Context**
```rust
invoke("rag_retrieve", {
    query_text: "what is X?",
    top_k: 10  // optional, default 5
})
→ Promise<Array<{id, score: float, content}>
```

**Index Directory**
```rust
invoke("rag_index", { path: "/path/to/files" })
→ Promise<string>  // "indexed"
```

### Management Commands

**Health Check**
```rust
invoke("rag_health_check")
→ Promise<{
    embeddings_ok: boolean,
    embeddings_error: string | null,
    llm_ok: boolean,
    llm_error: string | null
}>
```

**Get Stats**
```rust
invoke("rag_index_stats")
→ Promise<{
    indexed_documents: number,
    indexed_chunks: number,
    total_indexed_bytes: number,
    last_indexed_at: number | null,  // Unix timestamp
    estimated_index_size_mb: number
}>
```

**Clear Index**
```rust
invoke("rag_clear_index")
→ Promise<number>  // count of removed documents
```

## Configuration File

Location: `~/.nakama_config/rag_config.json`

```json
{
  "lancedb_path": "/home/user/.lancedb",
  "embedding_vector_size": 384,
  "embedding_model": "BAAI/bge-small-en-v1.5",
  "table_name": "nakama_documents",
  "chunk_size_min": 256,
  "chunk_size_max": 2048,
  "embed_batch_size": 8,
  "max_shard_size_mb": 500,
  "llm_backend_url": "http://localhost:11434",
  "retention_enabled": false,
  "retention_days": 90,
  "local_only": true
}
```

## TypeScript Interfaces

```typescript
interface RetrievedPassage {
  id: string;
  score: number;  // 0.0 to 1.0
  content: string;
}

interface HealthCheckResult {
  embeddings_ok: boolean;
  embeddings_error: string | null;
  llm_ok: boolean;
  llm_error: string | null;
}

interface IndexStats {
  indexed_documents: number;
  indexed_chunks: number;
  total_indexed_bytes: number;
  last_indexed_at: number | null;
  estimated_index_size_mb: number;
}
```

## Rust Types

```rust
// In src/commands/rag.rs
pub struct RetrievedPassage {
    pub id: String,
    pub score: f32,
    pub content: String,
}

pub struct HealthCheckResult {
    pub embeddings_ok: bool,
    pub embeddings_error: Option<String>,
    pub llm_ok: bool,
    pub llm_error: Option<String>,
}

pub struct IndexStats {
    pub indexed_documents: usize,
    pub indexed_chunks: usize,
    pub total_indexed_bytes: u64,
    pub last_indexed_at: Option<u64>,
    pub estimated_index_size_mb: u64,
}
```

## Common Usage Patterns

### 1. Upload & Use File Context
```typescript
// User uploads file in UI
await invoke("rag_add_file", { filename: "report.md", content });

// Later, in response generation
const context = await invoke<RetrievedPassage[]>(
    "rag_retrieve",
    { query_text: userQuery, top_k: 5 }
);

const augmentedPrompt = `
Context:
${context.map(p => p.content).join("\n---\n")}

User Question: ${userQuery}
`;
```

### 2. Verify System Health Before Indexing
```typescript
const health = await invoke<HealthCheckResult>("rag_health_check");
if (!health.embeddings_ok) {
  showError("Embedding engine not ready: " + health.embeddings_error);
  return;
}
if (!health.llm_ok) {
  showWarning("LLM backend offline: " + health.llm_error);
}
// Proceed with indexing...
```

### 3. Display Index Statistics
```typescript
const stats = await invoke<IndexStats>("rag_index_stats");
console.log(`
  Documents: ${stats.indexed_documents}
  Chunks: ${stats.indexed_chunks}
  Size: ${stats.estimated_index_size_mb}MB
  Last update: ${new Date(stats.last_indexed_at * 1000).toLocaleDateString()}
`);
```

### 4. Clear Index with Confirmation
```typescript
if (window.confirm("Delete all indexed data?")) {
  const removed = await invoke<number>("rag_clear_index");
  alert(`Removed ${removed} documents`);
}
```

## Debugging

### Enable Logging
```typescript
// In ChatContainer.tsx or other components
console.log("[RAG] Retrieving context...");
const retrieved = await invoke("rag_retrieve", { ... });
console.log("[RAG] Retrieved passages:", retrieved);
```

### Check LanceDB Directly
```bash
# On the system where Nakama runs
ls -la ~/.lancedb/

# If LanceDB tools installed:
lancedb ~/.lancedb
# > SELECT COUNT(*) FROM nakama_documents;
```

### Verify Ollama
```bash
curl http://localhost:11434/api/tags | jq
curl http://localhost:11434/api/health
```

## Build & Deploy

### Development
```bash
cd ~/Nakama
pnpm install
pnpm tauri dev --features swiftide_integration
```

### Production Build
```bash
pnpm tauri build --features swiftide_integration
# Output: src-tauri/target/release/bundle/
```

### Feature Flags
```toml
# src-tauri/Cargo.toml
[features]
default = ["swiftide_integration"]
swiftide_integration = []
```

## Performance Tips

| Scenario | Recommendation |
|----------|-----------------|
| Indexing large files (>10MB) | Reduce `embed_batch_size` to 4 |
| Slow retrieval | Enable vector compression (future) |
| Memory pressure | Enable retention policy with `retention_days: 30` |
| Many small files | Increase `embed_batch_size` to 16 |
| CPU-bound environments | Use CPU-optimized embedding model (future) |

## Error Handling

```typescript
try {
    const result = await invoke("rag_add_file", { filename, content });
    console.log(result);  // "file '...' indexed"
} catch (err) {
    console.error("Indexing failed:", err);
    // Display user-friendly message
    setMessage({
        type: "error",
        text: `Failed to index: ${String(err)}`
    });
}
```

## File Formats Supported

| Format | Status | Notes |
|--------|--------|-------|
| `.txt` | ✓ | Plain text, fully supported |
| `.md` | ✓ | Markdown, chunked intelligently |
| `.json` | ✓ | Parsed as text, structure preserved |
| `.csv` | ✓ | Converted to text format |
| `.pdf` | ◐ | Text extraction only (no images) |
| `.doc` / `.docx` | ✗ | Requires conversion to text first |
| `.png` / `.jpg` | ✗ | Vision models not yet supported |

---

**Last Updated**: February 26, 2026  
**Version**: 1.0  
**Keep handy for quick API lookup!**
