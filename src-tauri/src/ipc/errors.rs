// IPC errors.
//
// IPC error behavior must be explicit. Do not leak internal detail.

pub type IpcResult<T> = Result<T, crate::Error>;
