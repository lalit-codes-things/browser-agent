import type { HostChannel } from './client';

/**
 * Host attachment (Section 20.2).
 *
 * The managed host adapter installs exactly one channel on the app's own window
 * object. The channel is validated at runtime, field by field, before it is
 * used: a partially shaped or hostile object is refused rather than adapted, and
 * absence is reported as absence. Remote website frames never receive this
 * channel, and the check never treats a name being present as a capability.
 */

/** The property the host adapter installs on the app window. */
export const HOST_CHANNEL_PROPERTY = '__baHostChannel';

/** True when the value has the exact shape a host channel must have. */
export function isHostChannel(value: unknown): value is HostChannel {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as {
    hostLabel?: unknown;
    send?: unknown;
    subscribe?: unknown;
  };
  return (
    typeof candidate.hostLabel === 'string' &&
    candidate.hostLabel.length > 0 &&
    typeof candidate.send === 'function' &&
    typeof candidate.subscribe === 'function'
  );
}

/**
 * Returns the host channel, or `null` when no host has attached.
 *
 * `null` is not an error state to paper over: every component that consumes it
 * must report that the runtime is not connected instead of simulating one.
 */
export function attachHostChannel(scope: unknown = globalThis): HostChannel | null {
  if (typeof scope !== 'object' || scope === null) return null;
  const candidate = (scope as Record<string, unknown>)[HOST_CHANNEL_PROPERTY];
  return isHostChannel(candidate) ? candidate : null;
}
