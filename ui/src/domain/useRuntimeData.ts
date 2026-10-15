import { useMemo } from 'react';
import { commands } from '../bridge/client';
import { useRuntime } from './useRuntime';
import type { RuntimeSnapshot } from '../bridge/client';

/**
 * Commands and state for the secondary screens (Sections 2.8, 2.13, 17).
 *
 * A secondary screen shows what the runtime reported when it was asked. It never
 * caches a stale list, never shows a sample row and never renders an action whose
 * operation does not exist. Asking for data is a command; the answer arrives as
 * an event and is rendered from the snapshot.
 */

export interface RuntimeDataApi {
  snapshot: RuntimeSnapshot;
  requestHistory: () => void;
  clearHistory: () => void;
  requestNotes: () => void;
  createNote: (title: string, body: string, sourceUrl: string) => void;
  deleteNote: (note: string) => void;
  requestSkills: () => void;
  revokeSkill: (skill: string) => void;
  verifyAudit: () => void;
  requestModelStatus: () => void;
  installModel: (manifestId: string) => void;
  cancelModelInstall: () => void;
  verifyModel: () => void;
  resumeModel: () => void;
}

export function useRuntimeData(): RuntimeDataApi {
  const runtime = useRuntime();
  const client = runtime.client;

  return useMemo<RuntimeDataApi>(
    () => ({
      snapshot: runtime.snapshot,
      requestHistory: () => {
        client?.send(commands.lastTaskResult());
      },
      clearHistory: () => {
        client?.send(commands.clearTaskHistory());
      },
      requestNotes: () => {
        client?.send(commands.listNotes());
      },
      createNote: (title: string, body: string, sourceUrl: string) => {
        client?.send(commands.createNote(title, body, sourceUrl));
      },
      deleteNote: (note: string) => {
        client?.send(commands.deleteNote(note));
      },
      requestSkills: () => {
        client?.send(commands.listSkills());
      },
      revokeSkill: (skill: string) => {
        client?.send(commands.revokeSkill(skill));
      },
      verifyAudit: () => {
        client?.send(commands.verifyAudit());
      },
      requestModelStatus: () => {
        client?.send(commands.modelStatus());
      },
      installModel: (manifestId: string) => {
        client?.send(commands.modelInstall(manifestId));
      },
      cancelModelInstall: () => {
        client?.send(commands.modelCancelInstall());
      },
      verifyModel: () => {
        client?.send(commands.modelVerify());
      },
      resumeModel: () => {
        client?.send(commands.modelResume());
      },
    }),
    [client, runtime.snapshot],
  );
}
