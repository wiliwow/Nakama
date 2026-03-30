# In-Memory Document Indexing in Swiftide

## Overview
Swiftide allows indexing documents without writing them to disk using the `Pipeline::from_stream()` method with in-memory `IndexingStream` values.

## Key Types
- **`Node<T>`** or **`TextNode`** (= `Node<String>`): The fundamental unit in the indexing pipeline
- **`IndexingStream<T>`**: An async stream wrapper for `Result<Node<T>>` items
- **`Vec<Node<T>>`**: Automatically converts to `IndexingStream<T>` via `.into()`

## Pattern 1: Simple In-Memory Document (Recommended)

```rust
use swiftide::indexing::{Pipeline, Node};
use swiftide_core::Metadata;

#[tokio::main]
async fn index_single_document() -> Result<(), Box<dyn std::error::Error>> {
    // Create a document as a Node<String>
    let node = Node::builder()
        .chunk("Your document content here".to_string())
        .path("document.txt")  // Optional: path for reference
        .build()?;

    // Create pipeline from a vec of nodes
    let pipeline = Pipeline::from_stream(vec![node])
        .then_chunk(/* chunker */)
        .then_in_batch(Embed::new(/* embeddings */))
        .then_store_with(/* storage */);

    pipeline.run().await?;
    Ok(())
}
```

## Pattern 2: Multiple In-Memory Documents

```rust
use swiftide::indexing::{Pipeline, Node};

#[tokio::main]
async fn index_multiple_documents() -> Result<(), Box<dyn std::error::Error>> {
    let documents = vec![
        ("Document 1 content", "doc1.txt"),
        ("Document 2 content", "doc2.txt"),
        ("Document 3 content", "doc3.txt"),
    ];

    // Build nodes from your documents
    let nodes: Vec<Node<String>> = documents
        .into_iter()
        .map(|(content, path)| {
            Node::builder()
                .chunk(content.to_string())
                .path(path)
                .build()
                .expect("Failed to build node")
        })
        .collect();

    // Pass directly to Pipeline
    let pipeline = Pipeline::from_stream(nodes)
        .then_chunk(/* chunker */)
        .then_in_batch(Embed::new(/* embeddings */))
        .then_store_with(/* storage */);

    pipeline.run().await?;
    Ok(())
}
```

## Pattern 3: Using IndexingStream::iter() Explicitly

```rust
use swiftide::indexing::{Pipeline, Node, IndexingStream};

#[tokio::main]
async fn index_with_explicit_stream() -> Result<(), Box<dyn std::error::Error>> {
    // Create nodes
    let nodes = vec![
        Node::builder()
            .chunk("Content 1".to_string())
            .build()?,
        Node::builder()
            .chunk("Content 2".to_string())
            .build()?,
    ];

    // Explicitly create IndexingStream
    let stream = IndexingStream::from_nodes(nodes);

    let pipeline = Pipeline::from_stream(stream)
        .then_chunk(/* chunker */)
        .then_in_batch(Embed::new(/* embeddings */))
        .then_store_with(/* storage */);

    pipeline.run().await?;
    Ok(())
}
```

## Pattern 4: With Metadata (Nakama RAG Example - Fixed)

```rust
use swiftide::indexing::{Pipeline, Node};
use swiftide_core::Metadata;

#[tokio::main]
async fn index_with_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "uploaded_file.txt";
    let content = "Document content from upload";

    // Create a node with metadata
    let node = Node::builder()
        .chunk(content.to_string())
        .path(filename)
        .metadata(Metadata::from([("source", "upload")]))
        .build()?;

    let pipeline = Pipeline::from_stream(vec![node])
        .then_chunk(ChunkMarkdown::from_chunk_range(256..2048))
        .then_in_batch(Embed::new(fastembed).with_batch_size(8))
        .then_store_with(lancedb);

    match pipeline.run().await {
        Ok(_) => Ok(format!("file '{}' indexed", filename)),
        Err(e) => Err(format!("failed to index file: {}", e).into()),
    }
}
```

## Pattern 5: Converting from String to In-Memory Nodes

```rust
use swiftide::indexing::{Pipeline, Node};

#[tokio::main]
async fn index_string_content() -> Result<(), Box<dyn std::error::Error>> {
    let content = "This is my document content";
    
    // Direct conversion: String → Node → Pipeline
    let node = Node::builder()
        .chunk(content.to_string())
        .build()?;

    Pipeline::from_stream(vec![node])
        .then_chunk(ChunkMarkdown::from_chunk_range(10..2048))
        .then_in_batch(Embed::new(FastEmbed::try_default()?))
        .then_store_with(MemoryStorage::default())
        .run()
        .await?;

    Ok(())
}
```

## Pattern 6: Using tokio_stream for Iterator-Based Indexing

```rust
use swiftide::indexing::{Pipeline, Node};
use tokio_stream::iter;

#[tokio::main]
async fn index_with_tokio_stream() -> Result<(), Box<dyn std::error::Error>> {
    let documents = vec!["Doc 1", "Doc 2", "Doc 3"];

    // Convert docs to nodes
    let nodes = documents
        .into_iter()
        .map(|content| 
            Node::builder()
                .chunk(content.to_string())
                .build()
                .expect("Failed to build node")
        )
        .collect::<Vec<_>>();

    // Use tokio_stream for async iteration
    let stream = iter(nodes);

    Pipeline::from_stream(stream)
        .then_in_batch(Embed::new(FastEmbed::try_default()?))
        .then_store_with(MemoryStorage::default())
        .run()
        .await?;

    Ok(())
}
```

## Complete Example: Index Uploaded Files

```rust
use swiftide::{indexing, integrations};
use swiftide_core::indexing::Node;

pub async fn index_uploaded_content(
    filename: String,
    content: String,
) -> Result<String, String> {
    // Initialize embedder and storage
    let fastembed = integrations::fastembed::FastEmbed::try_default()
        .map_err(|e| format!("Failed to init embedder: {}", e))?;

    let lancedb = integrations::lancedb::LanceDB::builder()
        .table_name("documents".to_string())
        .build()
        .map_err(|e| format!("Failed to init storage: {}", e))?;

    // Create a node from uploaded content (not from disk!)
    let node = Node::builder()
        .chunk(content)
        .path(&filename)
        .build()
        .map_err(|e| format!("Failed to create node: {}", e))?;

    // Index this single node in-memory
    let pipeline = indexing::Pipeline::from_stream(vec![node])
        .then_chunk(
            indexing::transformers::ChunkMarkdown::from_chunk_range(256..2048)
        )
        .then_in_batch(
            indexing::transformers::Embed::new(fastembed).with_batch_size(8)
        )
        .then_store_with(lancedb);

    pipeline.run().await
        .map_err(|e| format!("Indexing failed: {}", e))?;

    Ok(format!("Successfully indexed '{}'", filename))
}
```

## Key APIs

### Creating a Node
```rust
// Using builder (recommended)
let node = Node::builder()
    .chunk(content)
    .path("file.txt")
    .metadata(metadata)
    .build()?;

// Using default + mutation
let mut node = Node::default();
node.chunk = "content".to_string();
```

### Creating IndexingStream
```rust
// From Vec<Node<T>> - automatic conversion
Pipeline::from_stream(vec![node1, node2])

// From Vec<Result<Node<T>>> - automatic conversion
let stream: Vec<Result<Node<String>>> = vec![Ok(node1)];
Pipeline::from_stream(stream)

// From IndexingStream explicitly
let stream = IndexingStream::from_nodes(vec![node1, node2]);
Pipeline::from_stream(stream)

// From iterator
let stream = IndexingStream::iter(vec![Ok(node1), Ok(node2)]);
```

### Pipeline Construction
```rust
Pipeline::from_stream(nodes)           // From in-memory nodes
Pipeline::from_loader(loader)          // From files/external source
Pipeline::from_stream(stream)          // Generic stream source
```

## Important Notes

⚠️ **Do NOT use swiftide_core::document::Document for indexing**
- `Document` is only used in the **query pipeline** (retrieving results)
- For **indexing**, always create `Node<String>` (= `TextNode`)
- `Document` doesn't implement `Chunk` trait needed by the pipeline

✓ **Use `Node<String>` for indexing**
- Indexing pipeline always works with `Node<T: Chunk>`
- String implements Chunk
- TextNode is an alias for Node<String>

## Complete Flow Diagram

```
String Content
    ↓
Node::builder().chunk(content)
    ↓
Vec<Node<String>>
    ↓
Pipeline::from_stream(vec)  [.into() Auto-converts]
    ↓
IndexingStream<String>
    ↓
.then_chunk()
    ↓
Processed Nodes
    ↓
.then_store_with()
    ↓
Stored/Indexed ✓
```
