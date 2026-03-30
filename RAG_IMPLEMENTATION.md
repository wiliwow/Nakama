# RAG Implementation in Nakama

## Overview
This document summarizes the RAG (Retrieval-Augmented Generation) implementation using Swiftide integrated with the Nakama Tauri desktop app.

## Architecture

### Backend (Rust/Tauri)
- **Vector Database:** LanceDB (local, file-based)
- **Embeddings:** FastEmbed (local, 384-dimensional dense vectors, no external API required)
- **Indexing Pipeline:** Swiftide `indexing::Pipeline`
- **Query Pipeline:** Swiftide `query::Pipeline`
- **Two Tauri Commands:**
  - `rag_index(path)` — indexes documents from a directory
  - `rag_retrieve(query, top_k)` — retrieves relevant passages

### Frontend (React/TypeScript)
- Invoke commands via `invoke('rag_index', { path })` and `invoke('rag_retrieve', { query_text, top_k })`
- Display retrieved passages to the user
- Combine retrieved context with LLM prompts for generation

## Files Modified / Created

### 1. `/home/wiliwow/Nakama/src-tauri/Cargo.toml`
- Added dependencies:
  ```toml
  swiftide = { path = "../../swiftide/swiftide", features = ["fastembed", "lancedb" ] }
  swiftide-core = { path = "../../swiftide/swiftide-core" }
  ```
- Added feature flag:
  ```toml
  [features]
  default = ["swiftide_integration"]
  swiftide_integration = []
  ```

### 2. `/home/wiliwow/Nakama/src-tauri/src/commands/mod.rs`
- Exposed new module:
  ```rust
  pub mod rag;
  ```

### 3. `/home/wiliwow/Nakama/src-tauri/src/commands/rag.rs` (NEW)
- Implements `rag_index()` and `rag_retrieve()` commands
- Uses LanceDB builder pattern (no `try_default()`)
- Pass-through answerer pattern for retrieval-only queries
- Returns `Vec<RetrievedPassage>` DTOs

### 4. `/home/wiliwow/Nakama/src-tauri/src/lib.rs`
- Imported RAG commands:
  ```rust
  #[cfg(feature = "swiftide_integration")]
  use commands::rag::{rag_index, rag_retrieve};
  ```
- Registered commands in `tauri::generate_handler![]` macro

## How It Works

### Indexing Flow
1. **Frontend:** User selects directory via file picker → calls `invoke('rag_index', { path })`
2. **Backend:**
   - `FileLoader` scans directory for files
   - `ChunkMarkdown` chunks content (256–2048 char chunks)
   - `FastEmbed` embeds each chunk to 384-dim vectors
   - `LanceDB` stores documents with vectors (table: `nakama_documents`)
3. **Result:** Indexed documents persisted locally

### Retrieval Flow
1. **Frontend:** User types query → calls `invoke('rag_retrieve', { query_text, top_k: 5 })`
2. **Backend:**
   - `QueryTransformer::Embed` embeds the query to 384-dim vector
   - `LanceDB` retrieves top-k most similar documents (similarity search)
   - Pass-through answerer converts `Retrieved` → `Answered` state (no LLM call here)
   - Extract document content and build `RetrievedPassage` DTOs
3. **Result:** Returns `Vec<RetrievedPassage>` with id, score, and content

### Example Frontend Usage

```typescript
import { invoke } from '@tauri-apps/api/core';

// Index documents
const indexResult = await invoke('rag_index', { path: '/path/to/docs' });
console.log(indexResult); // "indexed"

// Retrieve passages
const passages = await invoke('rag_retrieve', {
  query_text: 'How does Swiftide work?',
  top_k: 5,
});
console.log(passages); // [{ id, score, content }, ...]

// Combine with LLM for generation
const context = passages.map(p => p.content).join('\n---\n');
const prompt = `Context:\n${context}\n\nQuestion: ${question}`;
const answer = await invoke('ask_ai', { prompt });
```

## Key Design Decisions

1. **LanceDB over Qdrant:** Local, no external services required. Good for desktop apps.
2. **FastEmbed over OpenAI embeddings:** Local, no API keys, 384-dim model (Flag Embedding).
3. **Pass-through answerer:** Retrieval is decoupled from LLM generation (you call `ask_ai` separately).
4. **Builder pattern for LanceDB:** No `.try_default()` method; use builder for config.
5. **Feature flag:** `swiftide_integration` allows building Nakama without Swiftide if needed.

## Compilation Status

✅ **All files compile successfully** (with the dev profile):
```bash
cd /home/wiliwow/Nakama/src-tauri
cargo check --all-features
# Finished `dev` profile [unoptimized + debuginfo] target(s) in X.Xs
```

## Next Steps for Frontend Integration

1. **Create a RAG panel** in the React UI with:
   - File picker to select indexing directory
   - Search input field
   - Display retrieved passages with scores

2. **Integrate with chat flow:**
   - After user enters a query, call `rag_retrieve()`
   - Append retrieved context to the prompt sent to `ask_ai()`
   - Display both context and AI response

3. **Add index management:**
   - Show index status (indexed/empty)
   - Allow re-indexing or clearing the index

## Example React Component Skeleton

```typescript
import React, { useState } from 'react';
import { invoke, open } from '@tauri-apps/api';

export const RAGPanel = () => {
  const [indexPath, setIndexPath] = useState('');
  const [query, setQuery] = useState('');
  const [retrieved, setRetrieved] = useState<{ id: string; score: number; content: string }[]>([]);

  const handleIndex = async () => {
    const selected = await open({ directory: true });
    if (selected) {
      setIndexPath(selected);
      await invoke('rag_index', { path: selected });
    }
  };

  const handleRetrieve = async () => {
    const passages = await invoke('rag_retrieve', {
      query_text: query,
      top_k: 5,
    });
    setRetrieved(passages);
  };

  return (
    <div>
      <button onClick={handleIndex}>Index Directory</button>
      <p>Indexed: {indexPath || 'None'}</p>
      <input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search query..."
      />
      <button onClick={handleRetrieve}>Retrieve</button>
      <div>
        {retrieved.map((p) => (
          <div key={p.id}>
            <p><strong>Score:</strong> {p.score}</p>
            <p><pre>{p.content}</pre></p>
          </div>
        ))}
      </div>
    </div>
  );
};
```

## Troubleshooting

### Build Errors
- **"swiftide_core not found":** Ensure `swiftide-core` is in `Cargo.toml` dependencies.
- **"LanceDB builder error":** Verify the swiftide version uses builder pattern (not `::try_default()`).

### Runtime Errors
- **"swiftide integration not enabled":** Feature flag `swiftide_integration` not compiled in. Rebuild with `cargo build --all-features`.
- **Empty retrieval results:** Ensure documents were indexed first with `rag_index()`.

## References

- Swiftide docs: [https://github.com/bosun-ai/swiftide](https://github.com/bosun-ai/swiftide)
- LanceDB docs: [https://docs.lancedb.com](https://docs.lancedb.com)
- FastEmbed: [https://qdrant.github.io/fastembed/](https://qdrant.github.io/fastembed/)
- Tauri command invocation: [https://tauri.app](https://tauri.app)
