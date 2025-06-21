# MIRA Memory Architecture - RC0 (Release Candidate 0)

## Overview
Memory-Integrated Recursive Assistant (MIRA) - 6-tier hierarchical memory system with MCP integration for LLMs.

## Problem Statement
1. Users forget what they said previously
2. LLMs have context window limitations (~200K tokens)
3. Both struggle with "remember that thing we talked about?" queries
4. Need seamless memory without user awareness

## Architecture Options Explored

### Option 1: Traditional Tier System (L0-L5)
```
L0: Context Window (LLM internal, ~200K tokens)
L1: Session Memory (current session, no compression)
L2: Daily Memory (24hr, light compression)
L3: Weekly Memory (7d, concept extraction)
L4: Monthly Memory (30d, heavy compression)
L5: Permanent Memory (forever, extreme compression)
```
**Problem**: Artificial boundaries, compression = information loss

### Option 2: Inverted Storage Pyramid
```
L5: Raw Storage (S3/Files) - Everything, forever
L4: Indexed Layer - Searchable catalog
L3: Semantic Layer - Meaning network
L2: Active Context - Recent summaries
L1: Working Memory - Pre-loaded hot data
L0: Context Window - Current consciousness
```
**Insight**: Storage flows DOWN (gravity), Retrieval flows UP (attention)

### Option 3: Splay Tree Memory
```
- Single dynamic tree structure
- Frequently accessed memories rotate to root
- Natural hot/warm/cold distribution
- Semantic links cross-cutting the tree
```
**Problem**: Terrible for distributed systems, concurrency nightmare

## Current Consensus (Pragmatic Approach)

### 1. Storage Layer
```python
storage = {
    'postgres': 'All conversations, structured data',
    's3': 'Attachments, images, large files',
    'redis': 'Current session cache'
}
```

### 2. Index Layer
```python
indexes = {
    'embeddings_db': 'Semantic search (vector similarity)',
    'full_text': 'Keyword search (PostgreSQL FTS)',
    'entity_graph': 'Relationship tracking (Neo4j?)'
}
```

### 3. Retrieval Layer
```python
retrieval = {
    'hot_cache': 'Last N messages in Redis',
    'relevance_scorer': 'What to surface when',
    'context_builder': 'Natural injection into L0'
}
```

## MCP Interface for LLMs

### Core Functions
```python
# Store everything
memory.store({
    tier: 'auto',  # Let system decide
    data: {content, metadata},
    compression: 'none'  # Always keep original
})

# Smart retrieval
memory.retrieve({
    query: 'user question',
    strategy: 'semantic+temporal',
    limit: 10
})

# Update relevance
memory.update_relevance({
    memory_id: 'xyz',
    action: 'accessed|mentioned'
})

# Cross-reference
memory.cross_reference({
    memory_id: 'xyz',
    max_distance: 0.3
})
```

## Data Flow

### Input Processing
```
User Input → Store (immediate) → Index (async) → Ready for retrieval
           ↓
        LLM sees in context
           ↓
        Response → Store → Index
```

### Retrieval Flow
```
User Query → Relevance Engine → Multi-strategy Search
                               ↓
                        Semantic + Temporal + Entity
                               ↓
                        Ranked Results → Context Builder
                               ↓
                        Natural injection into conversation
```

## Key Insights

1. **No Information Loss**: Store everything raw, compress only for retrieval
2. **Multi-Strategy Search**: Semantic + keyword + entity + temporal
3. **Unconscious Operation**: User never knows memory is being managed
4. **Predictive Loading**: Anticipate what user might ask next
5. **Natural Surfacing**: "By the way, this relates to..."

## Open Questions

1. **Embeddings Model**: Which one for semantic search?
2. **Context Building**: How to naturally inject without seeming forced?
3. **Privacy**: User control over what's remembered?
4. **Multi-User**: How to handle shared vs private memories?
5. **Performance**: Latency targets for each operation?

## Next Steps

1. Build MVP with PostgreSQL + Redis only
2. Add semantic search with embeddings
3. Implement MCP interface
4. Test with real conversations
5. Iterate based on what actually helps users

---

Status: Design Phase
Date: 2025-06-18
Author: Elon + 지혁