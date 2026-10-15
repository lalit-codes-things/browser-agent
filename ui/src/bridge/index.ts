/**
 * Runtime bridge: the validated wire contract between the first-party UI and
 * the Rust application core (Section 20.2).
 *
 * - `protocol` mirrors the Rust `Command`/`EventPayload` schema and validates
 *   every inbound frame before application state can see it.
 * - `client` turns accepted frames into an immutable runtime snapshot and turns
 *   user intent into typed commands with their own sequence.
 * - `host` locates the host-provided transport, or reports that none exists.
 */
export * from './protocol';
export * from './client';
export * from './host';
