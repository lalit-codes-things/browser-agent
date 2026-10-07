// IPC error handling.
//
// Do not leak internal backend detail into the console. Surface a stable,
// operational message for actionable failures.
export function formatIpcError(error) {
    if (error && typeof error === "object" && "toString" in error) {
        return String(error.toString()).slice(0, 200);
    }
    return "Operation failed. Check the event stream for details.";
}
