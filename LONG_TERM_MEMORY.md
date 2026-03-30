# Long-Term Memory Integration Guide

This document describes the new long-term memory system for Nakama, enabling persistent local knowledge retention through RAG (Retrieval-Augmented Generation).

## Overview

The long-term memory system allows Nakama to:
- **Index documents locally** using FastEmbed embeddings and LanceDB vector storage
- **Retrieve relevant context** automatically when answering questions  
- **Maintain persistent local-only storage** with no cloud uploads
- **Manage memory** through a dedicated UI for viewing status and maintenance operations

## Architecture

### Components

**Backend (Rust)**
- `src-tauri/src/config.rs` - Configuration management for RAG settings
- `src-tauri/src/rag_indexer.rs` - Metadata store and indexing statistics
- `src-tauri/src/commands/rag.rs` - Tauri commands for indexing, retrieval, and health checks

**Frontend (React/TypeScript)**
- `src/components/IndexManager.tsx` - UI for index management and file uploads
- `src/components/ChatContainer.tsx` - Enhanced chat with automatic file indexing and retrieval

### Data Flow

```
User uploads files
       ↓
[Files staged in ChatContainer UI]
       ↓
User clicks "Index Files" OR sends message
       ↓
Backend: rag_add_file() → FastEmbed → LanceDB
       ↓
[Documents indexed and stored locally]
       ↓
User asks question
       ↓
Backend: rag_retrieve(query) → LanceDB similarity search → top-k passages
       ↓
Passages injected as context into LLM prompt
       ↓
LLM responds with context-aware answer
```

## Setup & Dependencies

### System Requirements
- **Ollama**: Running locally on `http://localhost:11434` for LLM inference
  - Install: https://ollama.ai
  - Run: `ollama run deepseek-r1:1.5b` (or another model)
- **FastEmbed**: Embedded in swiftide, requires Python environment for model download
- **LanceDB**: Local embedded vector database (no external service needed)

### Rust Build
```bash
cargo build --features swiftide_integration --manifest-path src-tauri/Cargo.toml
```

### Frontend Plugins
Ensure these Tauri plugins are available (check `src-tauri/Cargo.toml`):
- `tauri-plugin-fs` - File system access
- `tauri-plugin-dialog` - File picker dialog

## Usage

### 1. Check System Health
Open the Index Manager (📚 button, top-right) and click "Refresh" to verify:
- ✓ Embedding Engine: FastEmbed + LanceDB operational
- ✓ LLM Backend: Ollama reachable

If either shows ✗, check logs and ensure services are running.

### 2. Index Files
**Option A: Via ChatContainer**
1. Click the file attachment icon in MessageInput
2. Select one or multiple files (txt, md, json, csv, etc.)
3. Staged files appear in the chat UI
4. Click "Index Files" to index them to persistent memory
5. AI confirms indexed count

**Option B: Via Index Manager**
1. Click 📚 button → "Upload & Index Files"
2. Select files and upload
3. System automatically indexes to memory

### 3. Query with Context
1. (Optional) Index some files as above
2. Type a question related to indexed content
3. AI automatically retrieves relevant passages
4. LLM generates answer based on context (displays as one augmented prompt)

### 4. Manage Index
Via Index Manager modal:
- View current memory statistics (documents, chunks, size, last update)
- Upload and index new files
- Clear index (if needed to reset all memory)
- Run health checks

## Configuration

Default configuration is suitable for most users. To customize:

**Config File Location**: `~/.nakama_config/rag_config.json` (created on first use)

**Configurable Fields**:
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

## Advanced Features

### Retention Policies
Enable automatic deletion of old entries:
1. Set `retention_enabled: true` in config
2. Configure `retention_days` (default: 90 days)
3. Pruning runs automatically on app startup

### Chunking & Embedding
- **Chunk Size**: 256–2048 characters (configurable)
- **Overlap**: Maintained between chunks for context continuity
- **Batch Size**: 8 embeddings per batch (tune for GPU memory)
- **Vector Dim**: 384 (standard for BAAI/bge-small-en-v1.5)

### Supported File Formats
- `.txt` - Plain text
- `.md` - Markdown  
- `.json` - JSON documents
- `.csv` - CSV tables (parsed as text)
- `.pdf` - (Basic text extraction; requires PDF parsing)

## Troubleshooting

### "embeddings_ok: false" in health check
**Cause**: FastEmbed failed to initialize (missing model files or compatibility issue)
**Fix**:
- First run may download ~20MB embedding model to `~/.cache/huggingface/`
- Check network connectivity
- Clear cache: `rm -rf ~/.cache/huggingface/`

### "llm_ok: false" in health check
**Cause**: Ollama not running or not reachable on localhost:11434
**Fix**:
- Install Ollama: https://ollama.ai
- Start Ollama: `ollama run deepseek-r1:1.5b`
- Check: `curl http://localhost:11434/api/tags`

### Index grows but retrieval doesn't work
**Cause**: Query embeddings may not match indexed content well
**Fix**:
- Try more specific queries
- Rephrase to match indexed document language
- Check file format is text-readable

### Out of disk space
**Cause**: LanceDB grew too large (>50GB threshold)
**Fix**:
- Clear old entries: `retention_enabled: true` + `retention_days: 30`
- Clear index and re-index: Index Manager → Clear Index
- Compress vector storage (quantization—advanced)

## Performance Notes

- **First-time model download**: ~30–60 seconds (embedding model)
- **Indexing 1MB file**: ~2–5 seconds (embedding + storage)
- **Retrieval query**: ~200–500ms (embedding + LanceDB search)
- **Large batch indexing** (100+ files): Run in background; UI shows progress

## Security & Privacy

✓ **All data is stored locally** in `~/.lancedb` (no cloud uploads)
✓ **No API keys transmitted** (local Ollama + FastEmbed embeddings)
⚠️ **Encryption at rest**: Not yet supported (future enhancement)
⚠️ **Multi-user isolation**: Nakama instance is per-user (system-level isolation)

## API Reference

### Backend Commands (Tauri)

**rag_add_file** - Index a single file
```rust
invoke("rag_add_file", { filename: "doc.txt", content: "..." })
  → Result<String>  // e.g., "file 'doc.txt' indexed"
```

**rag_retrieve** - Retrieve top-k relevant passages
```rust
invoke("rag_retrieve", { query_text: "...", top_k: 10 })
  → Result<RetrievedPassage[]>  // { id, score, content, source?: String }
```

**rag_index** - Index all files in a directory
```rust
invoke("rag_index", { path: "/path/to/dir" })
  → Result<String>  // "indexed"
```

**rag_health_check** - Check system readiness
```rust
invoke("rag_health_check")
  → Result<HealthCheckResult>  // { embeddings_ok, llm_ok, errors }
```

**rag_clear_index** - Delete all indexed data
```rust
invoke("rag_clear_index")
  → Result<u32>  // count of removed documents
```

**rag_index_stats** - Get index statistics
```rust
invoke("rag_index_stats")
  → Result<IndexStats>  // { indexed_documents, size_mb, last_indexed_at }
```

## Testing & Validation

See `scripts/validate_memory_integration.sh` for automated validation:
```bash
./scripts/validate_memory_integration.sh
```

Manual test:
1. Enable Ollama: `ollama run deepseek-r1:1.5b`
2. Launch app: `pnpm tauri dev`
3. Open Index Manager → verify health status
4. Upload a sample text file
5. Ask a question related to the file
6. Verify answer includes relevant context

## Future Enhancements

- [ ] Encryption at rest for vector store
- [ ] Support for non-text files (images, audio transcripts)
- [ ] Vector compression for 50GB+ datasets
- [ ] Integration with remote vector DBs (Qdrant, Pinecone)
- [ ] Multi-user/multi-workspace support
- [ ] Advanced UI for vector similarity visualization
- [ ] Automatic periodic re-indexing

## Support

For issues, check:
1. Ollama is running: `curl http://localhost:11434/api/tags`
2. LanceDB is writable: `ls -la ~/.lancedb`
3. App logs: Browser console (F12) + terminal output
4. GitHub Issues: Submit with logs and reproduction steps

---

**Version**: 0.1.0  
**Last Updated**: February 2026
