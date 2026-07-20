# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Conventional Commits](https://www.conventionalcommits.org).

## [Unreleased]

### Refactor

- Use the published [`srs-sm2`](https://crates.io/crates/srs-sm2) crate (v0.1.0) for SM-2 spaced-repetition scheduling instead of the in-tree implementation. The review path in `review_view.rs` now calls `srs_sm2::schedule_next` directly, and `core/srs.rs` keeps only the tome-specific queue helpers (`review_header_copy`, `remove_card`, `HasId`).
