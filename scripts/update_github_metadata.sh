#!/usr/bin/env bash
# Update GitHub repository metadata
# Requires: gh CLI

set -euo pipefail

REPO="arclabs561/rank-rank"
DESCRIPTION="Information retrieval pipeline: retrieval, fusion, reranking, evaluation. Rust crates for RAG and vector search."
TOPICS="rust,information-retrieval,vector-search,rag,ranking,search"
WEBSITE="https://docs.rs/rank-retrieve"

gh repo edit "$REPO" --description "$DESCRIPTION"
gh repo edit "$REPO" --homepage "$WEBSITE"

for topic in $(echo "$TOPICS" | tr ',' ' '); do
    gh repo edit "$REPO" --add-topic "$topic" 2>/dev/null || true
done
